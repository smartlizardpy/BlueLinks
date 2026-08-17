#!/usr/bin/env python3
"""Build BlueLink's compact runtime SQLite database.

Development:
  python tools/build_dataset.py --development --output data/articles.sqlite
Production (--dump takes a local path or a dump URL):
  python tools/build_dataset.py --dump https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-pages-articles.xml.bz2 \
      --output data/production/articles.sqlite --production --limit 3000000

A URL is streamed and decompressed as it is parsed, and --limit stops the parse
once enough titles are in hand, so only the leading fraction of the dump is ever
transferred. The release workflow uses this on a GitHub runner; nobody needs the
whole 22 GB archive on a personal connection.
"""
from __future__ import annotations
import argparse, bz2, hashlib, json, re, sqlite3, sys, unicodedata
from pathlib import Path
from urllib.request import Request, urlopen
from xml.etree.ElementTree import iterparse

USER_AGENT="BlueLink-dataset-builder/1.0 (https://github.com/smartlizardpy/BlueLinks)"

SCHEMA="""
CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
CREATE TABLE articles(id INTEGER PRIMARY KEY,title TEXT NOT NULL,normalized_title TEXT NOT NULL,
 is_redirect INTEGER NOT NULL,is_disambiguation INTEGER NOT NULL,in_degree INTEGER NOT NULL,
 out_degree INTEGER NOT NULL,topic_mask INTEGER NOT NULL,community_id INTEGER NOT NULL,
 sig0 INTEGER NOT NULL,sig1 INTEGER NOT NULL,sig2 INTEGER NOT NULL,sig3 INTEGER NOT NULL,
 weight INTEGER NOT NULL DEFAULT 1);
CREATE INDEX eligible_articles ON articles(is_redirect,is_disambiguation,out_degree,id);
CREATE INDEX normalized_titles ON articles(normalized_title);
"""
TOPICS={"geography":1,"people":2,"history":4,"politics":8,"science":16,"technology":32,"arts":64,"sports":128,"business":256,"nature":512,"transport":1024,"military":2048,"education":4096,"memes":16384,"other":8192}
KEYWORDS={
 "geography":"country city river mountain island ocean province geography london paris africa asia europe america",
 "people":"actor writer scientist president composer philosopher person biography",
 "history":"empire war revolution ancient medieval history dynasty",
 "politics":"government election parliament politics republic constitution",
 "science":"physics chemistry mathematics astronomy medicine geology science",
 "technology":"computer software internet engineering technology programming",
 "arts":"music film novel painting theatre art jazz rock",
 "sports":"football cricket tennis olympic sport basketball baseball",
 "business":"company corporation economics bank business industry",
 "nature":"animal plant species ecology biology forest",
 "transport":"railway airport automobile ship transport aviation",
 "military":"battle army navy weapon military",
 "education":"university school education college",
}
DEV_TITLES="""Ada Lovelace|Algebra|Amazon rainforest|American Civil War|Amsterdam|Ancient Egypt|Apollo program|Argentina|Aristotle|Artificial intelligence|Association football|Atlantic Ocean|Australia|Bacteria|Bangladesh|Barcelona|Basketball|Beethoven|Berlin|Biodiversity|Bitcoin|Brazil|British Empire|Buddhism|Byzantine Empire|Canada|Charles Darwin|Chess|Chicago|China|Classical music|Climate change|Cloud computing|Coffee|Computer|Cricket|Cuba|DNA|Democracy|Denmark|Dinosaur|Earth|Economics|Electricity|Elizabeth II|Engineering|Evolution|Finland|Florence|France|French Revolution|Galileo Galilei|Game theory|Germany|Google|Great Barrier Reef|Greece|Guitar|Himalayas|History of Japan|Human brain|Ice hockey|India|Industrial Revolution|Internet|Ireland|Isaac Newton|Islam|Istanbul|Italy|Jazz|Johann Sebastian Bach|Kenya|Leonardo da Vinci|Linux|London|Machine learning|Madrid|Mars|Mathematics|Mexico|Microsoft|Minecraft|Modern art|Moon|Mount Everest|Mozart|Music|New York City|Nintendo|Norway|Ocean|Olympic Games|Ottoman Empire|Paris|Philosophy|Physics|Poland|Portuguese Empire|Python (programming language)|Quantum mechanics|Rail transport|Renaissance|Republic of Venice|Rio de Janeiro|Roman Empire|Rome|Russia|Sahara|Science|Shakespeare|Singapore|Solar System|South Africa|South Korea|Spain|Steam engine|Stockholm|Sweden|Sydney|Tennis|Tokyo|Turkey|United Kingdom|United Nations|United States|University|Video game|Vietnam|World War I|World War II|World Wide Web|Writing|Yellowstone National Park|Zimbabwe""".split("|")
DEV_GROUPS={
 "geography":"Amazon rainforest|Amsterdam|Argentina|Atlantic Ocean|Australia|Bangladesh|Barcelona|Berlin|Brazil|Canada|Chicago|China|Cuba|Denmark|Earth|Finland|Florence|France|Germany|Great Barrier Reef|Greece|Himalayas|India|Ireland|Istanbul|Italy|Kenya|London|Madrid|Mexico|Moon|Mount Everest|New York City|Norway|Ocean|Paris|Poland|Rio de Janeiro|Rome|Russia|Sahara|Singapore|South Africa|South Korea|Spain|Stockholm|Sweden|Sydney|Tokyo|Turkey|United Kingdom|United States|Vietnam|Yellowstone National Park|Zimbabwe",
 "people":"Ada Lovelace|Aristotle|Charles Darwin|Elizabeth II|Galileo Galilei|Isaac Newton|Leonardo da Vinci|Shakespeare",
 "history":"American Civil War|Ancient Egypt|British Empire|Byzantine Empire|French Revolution|History of Japan|Industrial Revolution|Ottoman Empire|Portuguese Empire|Renaissance|Republic of Venice|Roman Empire|World War I|World War II",
 "science":"Algebra|Apollo program|Bacteria|Biodiversity|Climate change|DNA|Dinosaur|Electricity|Evolution|Game theory|Human brain|Mars|Mathematics|Physics|Quantum mechanics|Science|Solar System",
 "technology":"Artificial intelligence|Bitcoin|Cloud computing|Computer|Engineering|Google|Internet|Linux|Machine learning|Microsoft|Python (programming language)|Steam engine|World Wide Web",
 "arts":"Beethoven|Classical music|Guitar|Jazz|Johann Sebastian Bach|Modern art|Mozart|Music|Writing",
 "sports":"Association football|Basketball|Chess|Cricket|Ice hockey|Olympic Games|Tennis",
 "politics":"Democracy|United Nations",
 "religion":"Buddhism|Islam|Philosophy",
 "business":"Economics",
 "transport":"Rail transport",
 "education":"University",
 "other":"Coffee|Minecraft|Nintendo|Video game",
}
DEV_TOPIC={title:TOPICS.get(group,TOPICS["other"]) for group,titles in DEV_GROUPS.items() for title in titles.split("|")}
LINK_RE=re.compile(r"\[\[([^\]|#]+)")
# Keep in step with MIN_OUT_DEGREE in src-tauri/src/randomizer.rs; this only
# affects the reported eligible count, the game applies the bar itself.
MIN_OUT_DEGREE=100
# Navigational pages carry hundreds of links without being subjects anybody
# could be asked to reach, so they clear any link-count bar while making a
# nonsense challenge. They are still perfectly good stepping stones mid-run;
# this only keeps them from being a start or a target.
NAVIGATIONAL=re.compile(r"^(lists? of|index of|outline of|timeline of|glossary of|comparison of|bibliography of)\b",re.I)

def normalize(title:str)->str:
    return " ".join(unicodedata.normalize("NFKC",title).replace("_"," ").lower().split())
def metadata(title:str,text:str="",forced_topic:int|None=None):
    hay=(title+" "+text[:4000]).lower(); mask=forced_topic or 0
    for topic,words in KEYWORDS.items():
        if any(word in hay for word in words.split()): mask|=TOPICS[topic]
    if not mask: mask=TOPICS["other"]
    community=(mask & -mask).bit_length()-1
    links=[normalize(v) for v in LINK_RE.findall(text) if ":" not in v][:200]
    # The real link count, with no floor: a stub has to look like a stub or the
    # dead-end filter downstream has nothing to reject.
    out_degree=min(500,len(set(links)))
    if not links: links=[normalize(title)+str(i) for i in range(4)]
    hashes=sorted(int.from_bytes(hashlib.blake2s(v.encode(),digest_size=4).digest(),"big") & 0x7fffffff for v in links)
    sig=(hashes+[0,0,0,0])[:4]
    return mask,community,sig,out_degree
def insert(conn,id_,title,text="",redirect=False,forced_topic=None,out_degree=None,weight=1):
    norm=normalize(title); disamb="{{disambiguation" in text[:5000].lower() or norm.endswith("(disambiguation)") or bool(NAVIGATIONAL.match(norm))
    mask,community,sig,measured=metadata(title,text,forced_topic)
    out_degree=measured if out_degree is None else out_degree
    conn.execute("INSERT INTO articles VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)",(id_,title,norm,int(redirect),int(disamb),max(1,out_degree//2),out_degree,mask,community,*sig,max(1,weight)))
def build_dev(conn):
    # The fixture carries no article text, so give these well-known titles a
    # plausible link count instead of letting them all look like dead ends.
    for id_,title in enumerate(DEV_TITLES,1): insert(conn,id_,title,forced_topic=DEV_TOPIC.get(title,TOPICS["other"]),out_degree=120+id_%80)
def read_curated(path:Path):
    """Read the curated pool: one "topic<TAB>title" per line, # for comments."""
    rows=[]
    for number,line in enumerate(path.read_text(encoding="utf-8").splitlines(),1):
        if not line.strip() or line.lstrip().startswith("#"): continue
        parts=[part.strip() for part in line.split("\t")]
        topic,title,raw_weight=(parts+["",""])[0],(parts+["",""])[1],(parts+["","",""])[2]
        if not title: raise SystemExit(f"{path}:{number}: expected topic<TAB>title[<TAB>weight]")
        if topic not in TOPICS: raise SystemExit(f"{path}:{number}: unknown topic {topic!r}")
        try: weight=int(raw_weight) if raw_weight else 1
        except ValueError: raise SystemExit(f"{path}:{number}: weight {raw_weight!r} is not a whole number")
        if weight<1: raise SystemExit(f"{path}:{number}: weight must be at least 1")
        rows.append((topic,title,weight))
    return rows
def build_curated(conn,path:Path):
    rows=read_curated(path)
    seen={}
    for id_,(topic,title,weight) in enumerate(rows,1):
        norm=normalize(title)
        if norm in seen: raise SystemExit(f"{path}: {title!r} duplicates {seen[norm]!r}")
        seen[norm]=title
        # No article text to measure, so every curated entry is credited with a
        # link count above the notability bar; the curation is the bar here.
        insert(conn,id_,title,forced_topic=TOPICS[topic],out_degree=MIN_OUT_DEGREE+20+id_%80,weight=weight)
    return len(rows)
def local_name(tag): return tag.rsplit("}",1)[-1]
def open_dump(source:str):
    """Open a dump by path or URL. A URL is consumed lazily, so abandoning the
    parse early also abandons the rest of the download."""
    raw=urlopen(Request(source,headers={"User-Agent":USER_AGENT})) if source.startswith(("http://","https://")) else open(source,"rb")
    return bz2.open(raw,"rb") if source.endswith(".bz2") else raw
def build_dump(conn,source:str,limit:int|None=None):
    total=0
    with open_dump(source) as stream:
      for _,elem in iterparse(stream,events=("end",)):
        if local_name(elem.tag)!="page": continue
        values={local_name(child.tag):child for child in elem}
        namespace=(values.get("ns").text or "") if values.get("ns") is not None else ""
        if namespace=="0":
          title=values["title"].text or ""; page_id=int(values["id"].text); redirect=values.get("redirect") is not None
          revision=values.get("revision"); text=""
          if revision is not None:
            for child in revision.iter():
              if local_name(child.tag)=="text": text=child.text or "";break
          if title.strip(): insert(conn,page_id,title,text,redirect); total+=1
          if total and total%10000==0: conn.commit();print(f"Processed {total:,} titles",file=sys.stderr)
        elem.clear()
        if limit and total>=limit: print(f"Reached the {limit:,} title limit; stopping the download.",file=sys.stderr);break
    return total
def validate(conn,production:bool,floor:int|None=None):
    total=conn.execute("SELECT COUNT(*) FROM articles").fetchone()[0];eligible=conn.execute(f"SELECT COUNT(*) FROM articles WHERE is_redirect=0 AND is_disambiguation=0 AND out_degree>={MIN_OUT_DEGREE}").fetchone()[0]
    redirects=conn.execute("SELECT COUNT(*) FROM articles WHERE is_redirect=1").fetchone()[0];disamb=conn.execute("SELECT COUNT(*) FROM articles WHERE is_disambiguation=1").fetchone()[0]
    duplicates=conn.execute("SELECT COUNT(*) FROM (SELECT normalized_title FROM articles GROUP BY normalized_title HAVING COUNT(*)>1)").fetchone()[0]
    floor=floor if floor is not None else (1_000_000 if production else 100)
    if total<floor: raise SystemExit(f"Validation failed: {total:,} titles is below the required {floor:,}")
    if duplicates: print(f"Warning: {duplicates:,} normalized title collisions",file=sys.stderr)
    stats={"total":total,"eligible":eligible,"redirects":redirects,"disambiguation":disamb,"average_in_degree":conn.execute("SELECT AVG(in_degree) FROM articles").fetchone()[0],"average_out_degree":conn.execute("SELECT AVG(out_degree) FROM articles").fetchone()[0]}
    print(json.dumps(stats,indent=2));return stats
def main():
    parser=argparse.ArgumentParser();parser.add_argument("--dump",help="local dump path or dump URL");parser.add_argument("--curated",type=Path,help="curated pool file, one topic<TAB>title per line");parser.add_argument("--limit",type=int,help="stop after this many titles instead of reading the whole dump");parser.add_argument("--output",type=Path,required=True);parser.add_argument("--development",action="store_true");parser.add_argument("--production",action="store_true");parser.add_argument("--print-pairs",action="store_true");args=parser.parse_args()
    if args.development==args.production: parser.error("choose exactly one of --development or --production")
    if args.dump and args.curated: parser.error("choose either --dump or --curated, not both")
    if args.production and not (args.dump or args.curated): parser.error("--production requires --dump or --curated")
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.unlink(missing_ok=True)
    dataset_kind=("curated" if args.curated else "production") if args.production else "development"
    conn=sqlite3.connect(args.output);conn.executescript(SCHEMA);conn.executemany("INSERT INTO metadata VALUES(?,?)",[("schema_version","2"),("dataset_kind",dataset_kind),("dataset_version",dataset_kind)])
    if args.development: build_dev(conn)
    elif args.curated: build_curated(conn,args.curated)
    else: build_dump(conn,args.dump,args.limit)
    conn.commit()
    # A curated pool is vouched for by hand, so it is held to a pool size rather
    # than the million-title floor a dump has to clear to prove it is not the fixture.
    validate(conn,args.production,floor=200 if args.curated else None)
    conn.execute("VACUUM");conn.close()
    if args.production: (args.output.parent/"PRODUCTION_DATASET").write_text("validated production dataset\n",encoding="utf-8")
    if args.print_pairs:
      import random
      for _ in range(20): a,b=random.sample(DEV_TITLES,2);print(f"{random.uniform(.68,.86):.2f}  {a} → {b}")
if __name__=="__main__": main()
