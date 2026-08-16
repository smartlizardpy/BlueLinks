import { mkdir, writeFile } from "node:fs/promises";

const repository = process.env.GITHUB_REPOSITORY;
const publicKey = process.env.TAURI_UPDATER_PUBKEY?.trim();

// The production database is bundled either way. Updater artifacts are only
// meaningful when a public key is present to verify them, so a build without
// one produces a plain installer instead of a self-updating release.
const config = {
  bundle: { resources: { "../data/production/articles.sqlite": "articles.sqlite" } },
};

if (publicKey) {
  if (!repository || !/^[^/]+\/[^/]+$/.test(repository)) {
    throw new Error("GITHUB_REPOSITORY must contain OWNER/REPOSITORY.");
  }
  config.bundle.createUpdaterArtifacts = true;
  config.plugins = {
    updater: {
      pubkey: publicKey,
      endpoints: [`https://github.com/${repository}/releases/latest/download/latest.json`],
      windows: { installMode: "passive" },
    },
  };
}

await mkdir(".release", { recursive: true });
await writeFile(".release/tauri.release.conf.json", `${JSON.stringify(config, null, 2)}\n`, { mode: 0o600 });
console.log(
  publicKey
    ? `Generated signed-updater release config for ${repository}.`
    : "Generated release config with the production database; updating is disabled without TAURI_UPDATER_PUBKEY.",
);
