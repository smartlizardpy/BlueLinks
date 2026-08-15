import { mkdir, writeFile } from "node:fs/promises";

const repository = process.env.GITHUB_REPOSITORY;
const publicKey = process.env.TAURI_UPDATER_PUBKEY;
if (!repository || !/^[^/]+\/[^/]+$/.test(repository)) {
  throw new Error("GITHUB_REPOSITORY must contain OWNER/REPOSITORY.");
}
if (!publicKey?.trim()) {
  throw new Error("TAURI_UPDATER_PUBKEY repository variable is missing.");
}

const config = {
  bundle: {
    createUpdaterArtifacts: true,
    resources: { "../data/production/articles.sqlite": "articles.sqlite" },
  },
  plugins: {
    updater: {
      pubkey: publicKey.trim(),
      endpoints: [`https://github.com/${repository}/releases/latest/download/latest.json`],
      windows: { installMode: "passive" },
    },
  },
};
await mkdir(".release", { recursive: true });
await writeFile(".release/tauri.release.conf.json", `${JSON.stringify(config, null, 2)}\n`, { mode: 0o600 });
console.log(`Generated signed-updater release config for ${repository}.`);
