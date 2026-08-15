# BlueLink

BlueLink is a Windows-first Wikipedia navigation speedrun game. It generates a local start/target challenge, places the player in the real English Wikipedia website, and stops a monotonic timer when the target article is reached. Challenge generation, scoring, run state, personal bests, and recent history remain local; only article browsing needs the internet.

The finished game includes Normal, Max Clicks, Time Limit, Fewest Clicks, Speedrun, Evil, pass-the-PC 2 Player, and five-stage Gauntlet modes. Settings, exact-pair personal bests, streaks, and the latest 100 runs are persisted locally. The embedded Wikipedia view allows only English main-namespace article navigation and counts one completed article transition as one click, including redirects.

The current Windows installer is [release/BlueLink-Windows-Setup.exe](release/BlueLink-Windows-Setup.exe).

## Development

Prerequisites: Node.js 20+, Rust stable, the platform dependencies listed in the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/), and Python 3 only when rebuilding article data.

```bash
npm install
python tools/build_dataset.py --development --output data/articles.sqlite
npm run tauri dev
```

Run all frontend and Rust checks:

```bash
npm run check
```

The development database contains a varied curated set of namespace-0 titles. It is deliberately marked as development data.

## Production article data

Download an English Wikipedia `pages-articles` XML dump from <https://dumps.wikimedia.org/enwiki/latest/> into a local dump directory, then run:

```bash
python tools/build_dataset.py \
  --dump dumps/enwiki-latest-pages-articles.xml.bz2 \
  --output data/production/articles.sqlite \
  --production
```

The tool streams namespace-0 pages, normalizes titles, detects redirects/disambiguation pages, calculates compact topic/link-neighborhood metadata, validates counts and ranges, prints statistics, and creates a `PRODUCTION_DATASET` marker. Python is not used by the installed game.

For richer production scoring, the database schema is designed to accept more accurate in/out degree, community, topic, and MinHash-style signature values derived from Wikimedia pagelinks/category dumps. The runtime interface does not change when those columns are enriched.

## Windows release

Production packaging intentionally refuses to fall back to the toy database:

```powershell
$env:BLUELINK_PRODUCTION="1"
npm run tauri build
```

The build guard requires both `data/production/articles.sqlite` and its validation marker. Tauri produces normal NSIS/MSI artifacts in `src-tauri/target/release/bundle`. Node, Python, and a BlueLink server are not required after installation. Wikipedia navigation still requires internet access.

Signed application updates use Tauri's official updater and a deliberate `vX.Y.Z` GitHub tag workflow. See [docs/RELEASING.md](docs/RELEASING.md) for the signing-key, repository variable, production dataset, and release procedure. Unsigned local development builds intentionally leave update checking unavailable rather than embedding a fake key.

## Architecture

The trusted local React WebView owns only the yellow-and-black Start, Game header, and Result screens. During a run Rust creates a separate child WebView for `en.wikipedia.org` below the 56px game bar. That remote WebView is absent from the shell capability target and receives no privileged command bridge.

Rust owns challenge selection, URL/namespace policy, navigation transactions, canonical route recording, the monotonic timer, target detection, personal bests, and the capped 100-run history. React owns rendering, scramble animations, and smooth timer interpolation. Each run has a unique ID so stale WebView callbacks cannot affect a later run.

The randomizer rejects redirects, disambiguation pages, dead ends, lexical near-duplicates, and metadata-indicated direct conceptual neighbors. It combines lexical, topic, graph-signature, community, popularity-balance, and navigability signals and selects from a hard-but-fair difficulty band.

See [data/ATTRIBUTION.md](data/ATTRIBUTION.md) for Wikimedia data attribution.
