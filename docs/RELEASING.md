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

## Preview builds

`Playable Windows build` (`.github/workflows/playable.yml`) produces a downloadable installer without any signing secrets. Run it from the Actions tab, choosing the production or development database, and it attaches the installers to a `preview-<run>` prerelease as well as to the run's artifacts. Preview builds are unsigned and have no updater, so Windows warns before running them and they cannot upgrade themselves; everything else about the game is identical to a release build. Note that a prerelease does not answer `/releases/latest`, so the README download link stays dark until a real tag ships.

## The article database

The release job builds `data/production/articles.sqlite` on the runner. `tools/build_dataset.py` reads the dump straight from `dumps.wikimedia.org` and parses it while it arrives, then stops at `DATASET_ARTICLE_LIMIT` titles and drops the connection. Only the leading fraction of the archive is ever transferred, and it is transferred over GitHub's connection rather than yours.

The result is cached under the key `bluelink-dataset-<DATASET_REVISION>-<DATASET_ARTICLE_LIMIT>`, so only the first release after a change pays for the build. Both values are job-level `env` entries in `.github/workflows/release.yml`:

- `DATASET_REVISION` — bump it to pick up a fresher dump.
- `DATASET_ARTICLE_LIMIT` — how many titles to keep. `tools/build_dataset.py` rejects anything under 1,000,000 for a production build, and every extra title makes the shipped installer larger.

Builds that carry the production database ship NSIS only. WiX's `light.exe` cannot build a cabinet around a database this size and fails the bundle step; NSIS packages the same payload in about two minutes, and the updater already prefers the NSIS artifact. Development builds still produce both an `.exe` and an `.msi`.

Because the database is generated, it is gitignored and must not be committed. `src-tauri/build.rs` still refuses to package when `BLUELINK_PRODUCTION=1` and the database or its `PRODUCTION_DATASET` marker is absent, so a release can never quietly ship the small development fixture.

You do not need a local production database to cut a release. If you want one anyway, and you are not on a metered connection:

```powershell
python tools/build_dataset.py --dump https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-pages-articles.xml.bz2 --output data/production/articles.sqlite --production --limit 2000000
```

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

The published app checks `https://github.com/OWNER/REPOSITORY/releases/latest/download/latest.json`, with the real repository path injected by GitHub Actions. Update signing is separate from optional Windows Authenticode code signing; add a trusted Windows certificate before broad distribution to reduce SmartScreen warnings.
