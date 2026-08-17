# Releasing BlueLink for Windows

BlueLink releases are built on GitHub's Windows runner. The release job creates the NSIS/MSI installers, signed updater bundles and signatures, and the official `latest.json`. The app's endpoint is generated from `GITHUB_REPOSITORY`, so no owner/repository placeholder is committed.

## One-time setup

1. Generate a Tauri updater key pair on a trusted computer:

   ```powershell
   npm run tauri signer generate -- -w $env:APPDATA\BlueLink\updater.key
   ```

   Keep `updater.key` and its password outside this repository and in a backed-up password manager. Losing the private key prevents future installations from accepting updates.

2. In the GitHub repository, create these Actions secrets:

   - `TAURI_SIGNING_PRIVATE_KEY`: the complete private key content
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: its password

3. Create one repository Actions variable named `TAURI_UPDATER_PUBKEY` containing the complete generated public key. This key is public by design and is embedded into release builds.

There is no fourth step: the production article database is built by the release job itself. You never download a Wikipedia dump, and the database is never committed.

Never print the private key or password. Never commit `.env`, `*.key`, the generated `.release` directory, or password files.

## Unsigned builds

`Playable Windows build` (`.github/workflows/playable.yml`) produces a downloadable installer without any signing secrets. Run it from the Actions tab, choosing the production or development database, and it attaches the installers to a `build-<run>` release as well as to the run's artifacts. These builds are unsigned and have no updater, so Windows warns before running them and they cannot upgrade themselves; everything else about the game is identical to a signed release.

Whether they publish as a release or a prerelease is decided per run, and that choice is load-bearing rather than cosmetic:

- **No signed `vX.Y.Z` release exists.** The build publishes as the latest release. There is nothing for it to displace, and it means the download links resolve instead of pointing at an empty `/releases/latest`.
- **A signed release exists.** The build publishes as a prerelease. The updater asks `/releases/latest/download/latest.json` and only a signed release carries `latest.json`, so an unsigned build taking that slot would 404 the endpoint underneath everybody who already installed the game. A prerelease can never become "latest".

The switch happens on its own, by checking whether any tag matching `vX.Y.Z` has a release. Nobody has to remember to flip it when the first signed version ships.

Each run also deletes its superseded builds, keeping the newest `KEEP_BUILDS` (3) and removing the tags with them. Only `build-<n>` and `preview-<n>` are considered; a signed `vX.Y.Z` release cannot match the pattern and is never touched. This is worth having rather than tidiness: a stale build left lying around can inherit `/releases/latest` the moment a newer release is deleted, and hand players a version whose bugs were fixed long ago.

The `build-<run>` tag is deliberately not of the form `vX.Y.Z`, so it does not trigger the signed release workflow. The website resolves prereleases by itself, so the download button works regardless; the README links to the Releases page rather than `/releases/latest`, which stays dark until a signed release exists.

## The article database

The shipped database is a hand-picked pool, `data/curated.tsv`, one `topic<TAB>title` per line. Every entry is an article people have heard of, which is the whole point: the game is only enjoyable when both ends of a challenge are recognisable, and no statistical filter over all of Wikipedia reliably delivers that.

Add or remove lines freely. Two rules:

- The title must be the **canonical** article title, not a redirect. A run detects arrival by comparing titles, so a redirect target can never be reached and the run can never be won.
- The topic must be one of the keys in `TOPICS` in `tools/build_dataset.py`: geography, people, history, politics, science, technology, arts, sports, business, nature, transport, military, education, memes, other. Topic is not decoration — the selector rejects a pair whose two articles share a single topic, so grouping a set of articles under their own topic makes them pair outward into the rest of the pool rather than with each other.

An optional third column is a weight: how many times more likely that article is to be drawn as a start or a target than a plain entry. Leave it off for 1. Weights apply to both ends of a challenge, so a weight of 6 makes an article roughly six times as likely to turn up at either end.

`tools/verify_titles.py` checks the whole pool against the Wikipedia API — missing articles, redirects, disambiguation pages and duplicates — and prints the canonical form of anything that needs fixing. Both workflows run it before building, so a pool that cannot be vouched for never ships.

```powershell
python tools/verify_titles.py data/curated.tsv
python tools/build_dataset.py --curated data/curated.tsv --output data/production/articles.sqlite --production
```

To see what the pool actually produces, without installing anything:

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --example print_pairs -- ../data/articles.sqlite 20
```

`tools/build_dataset.py --dump` still exists and still streams a Wikipedia dump for anyone who wants a database of millions of titles. Nothing in CI uses it: it made a 77 MB installer, cost half an hour of dump parsing per rebuild, and produced challenges like a minor novel to a road tunnel.

Because the database is generated, it is gitignored and must not be committed. `src-tauri/build.rs` still refuses to package when `BLUELINK_PRODUCTION=1` and the database or its `PRODUCTION_DATASET` marker is absent.

Builds that carry the production database ship NSIS only. WiX's `light.exe` cannot build a cabinet around a large database and fails the bundle step; NSIS packages the same payload in about two minutes, and the updater already prefers the NSIS artifact.

## Publish a version

Keep the version in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` synchronized. Validate it locally:

```powershell
npm run version:check
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Commit the version bump, then create a matching tag:

```powershell
git tag v1.0.1
git push origin main
git push origin v1.0.1
```

Installers are produced by GitHub Actions and attached to the GitHub release. Never commit a built `.exe` or `.msi` to the repository: a local installer embeds whichever dataset was present when it was built, and it silently goes stale as soon as the next commit lands.

Only matching `vX.Y.Z` tags publish releases. The workflow also checks that the tag exactly matches the application version and fails clearly when a signing secret, public key, or production dataset is missing.

## What makes updating work

Automatic updates need all four of these, and the first is the only one still outstanding:

1. The signing secrets and `TAURI_UPDATER_PUBKEY` from the one-time setup above.
2. `src-tauri/src/lib.rs` only registers the updater plugin when `TAURI_UPDATER_PUBKEY` is compiled in, which `release.yml` sets and `playable.yml` does not. An unsigned build has no updater at all, which is why the Settings screen reports that it cannot check.
3. `tools/write_release_config.mjs` only emits `createUpdaterArtifacts` and the `plugins.updater` block when that key is present, so only signed builds produce `latest.json` and the accompanying signature.
4. Nothing unsigned may answer `/releases/latest`, which is why preview builds stay prereleases.

Once the secrets exist, a `vX.Y.Z` tag publishes a signed release with `latest.json`, and installed copies pick it up on the next launch. Nothing else needs changing.

The published app checks `https://github.com/OWNER/REPOSITORY/releases/latest/download/latest.json`, with the real repository path injected by GitHub Actions. Update signing is separate from optional Windows Authenticode code signing; add a trusted Windows certificate before broad distribution to reduce SmartScreen warnings.
