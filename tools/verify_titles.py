#!/usr/bin/env python3
"""Check every curated title against Wikipedia.

  python tools/verify_titles.py data/curated.tsv

A run detects arrival by comparing the visited title with the target title, so a
redirect in the pool is a target that can never be reached and a run that can
never be won. This asks the API which titles exist and which are redirects, and
prints the canonical form for anything that needs fixing.

Exits non-zero if any title is missing, a redirect, or a disambiguation page, so
CI can refuse to ship a pool it cannot vouch for.
"""
from __future__ import annotations
import argparse, json
from pathlib import Path
from urllib.parse import urlencode
from urllib.request import Request, urlopen

API = "https://en.wikipedia.org/w/api.php"
USER_AGENT = "BlueLink-title-verifier/1.0 (https://github.com/smartlizardpy/BlueLinks)"
BATCH = 50


def read_pool(path: Path) -> list[str]:
    titles = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        # topic, title, then an optional weight.
        parts = [part.strip() for part in line.split("\t")]
        if len(parts) >= 2 and parts[1]:
            titles.append(parts[1])
    return titles


def query(titles: list[str]) -> dict:
    params = {
        "action": "query",
        "titles": "|".join(titles),
        "prop": "pageprops",
        "ppprop": "disambiguation",
        "format": "json",
        "formatversion": "2",
    }
    request = Request(f"{API}?{urlencode(params)}", headers={"User-Agent": USER_AGENT})
    with urlopen(request, timeout=30) as response:
        return json.load(response)


def apply_fixes(path: Path, renames: dict[str, str], drop: set[str]) -> None:
    """Rewrite redirects to their canonical title and comment out the rest.

    Whoever adds to the pool usually cannot check it as they write, so the
    check is more useful if it can also do the correcting.
    """
    out = []
    for line in path.read_text(encoding="utf-8").splitlines():
        parts = [part.strip() for part in line.split("\t")]
        title = parts[1] if len(parts) >= 2 and not line.lstrip().startswith("#") else None
        if title in renames:
            parts[1] = renames[title]
            out.append("\t".join(parts))
        elif title in drop:
            out.append(f"# unverified, needs a canonical title: {line}")
        else:
            out.append(line)
    path.write_text("\n".join(out) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("pool", type=Path, nargs="?", default=Path("data/curated.tsv"))
    parser.add_argument("--fix", action="store_true", help="rewrite redirects and comment out unusable entries")
    args = parser.parse_args()

    titles = read_pool(args.pool)
    duplicates = {t for t in titles if titles.count(t) > 1}
    problems: list[str] = []
    renames: dict[str, str] = {}
    drop: set[str] = set()
    if duplicates:
        problems += [f"duplicate entry: {t}" for t in sorted(duplicates)]

    for start in range(0, len(titles), BATCH):
        batch = titles[start : start + BATCH]
        data = query(batch)
        # Redirects are reported as a mapping rather than on the page itself,
        # because the query resolves them only when asked to.
        for page in data.get("query", {}).get("pages", []):
            title = page.get("title", "")
            if page.get("missing"):
                problems.append(f"no such article: {title}")
                drop.add(title)
            elif "disambiguation" in (page.get("pageprops") or {}):
                problems.append(f"disambiguation page: {title}")
                drop.add(title)

    # A second pass with redirect resolution turned on names the canonical form.
    for start in range(0, len(titles), BATCH):
        batch = titles[start : start + BATCH]
        params = {
            "action": "query",
            "titles": "|".join(batch),
            "redirects": "1",
            "format": "json",
            "formatversion": "2",
        }
        request = Request(f"{API}?{urlencode(params)}", headers={"User-Agent": USER_AGENT})
        with urlopen(request, timeout=30) as response:
            data = json.load(response)
        for redirect in data.get("query", {}).get("redirects", []):
            problems.append(f"redirect: {redirect['from']} -> use {redirect['to']}")
            renames[redirect["from"]] = redirect["to"]

    print(f"Checked {len(titles)} curated titles.", flush=True)
    if problems:
        for problem in sorted(set(problems)):
            print(f"  {problem}")
        print(f"{len(set(problems))} problem(s) found.", flush=True)
        if args.fix:
            apply_fixes(args.pool, renames, drop)
            print(f"Rewrote {len(renames)} redirect(s) and commented out {len(drop)} entry/entries in {args.pool}.")
            print("Re-run without --fix to confirm.")
        return 1
    print("Every title is a canonical article.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
