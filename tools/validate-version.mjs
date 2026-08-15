import { readFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const packageJson = JSON.parse(await readFile(new URL("package.json", root), "utf8"));
const tauriConfig = JSON.parse(await readFile(new URL("src-tauri/tauri.conf.json", root), "utf8"));
const cargo = await readFile(new URL("src-tauri/Cargo.toml", root), "utf8");
const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const versions = { package: packageJson.version, tauri: tauriConfig.version, cargo: cargoVersion };

if (!versions.package || new Set(Object.values(versions)).size !== 1) {
  throw new Error(`Version mismatch: ${JSON.stringify(versions)}`);
}
const tag = process.argv[2];
if (tag && tag !== `v${versions.package}`) {
  throw new Error(`Release tag ${tag} does not match application version v${versions.package}`);
}
console.log(`BlueLink version ${versions.package} is synchronized.`);
