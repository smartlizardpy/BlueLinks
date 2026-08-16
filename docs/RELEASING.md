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

4. Install Git LFS, build and validate a production article database as described in the README, then commit both production files:

   ```powershell
   git lfs install
   git add data/production/articles.sqlite data/production/PRODUCTION_DATASET
   git commit -m "data: add production Wikipedia dataset"
   git push origin main
   ```

   The SQLite file is already configured for Git LFS, and the release workflow downloads LFS objects during checkout. The release build deliberately fails if the database or marker is missing, so it cannot accidentally ship the tiny development fixture.

Never print the private key or password. Never commit `.env`, `*.key`, the generated `.release` directory, or password files.

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
