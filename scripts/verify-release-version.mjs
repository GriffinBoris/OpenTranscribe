import { readFileSync } from "node:fs";
import process from "node:process";

const tag = process.env.GITHUB_REF_NAME;
const packageVersion = JSON.parse(readFileSync("package.json", "utf8")).version;
const expectedVersion = tag?.startsWith("v") ? tag.slice(1) : packageVersion;
const tauriVersion = JSON.parse(
  readFileSync("src-tauri/tauri.conf.json", "utf8"),
).version;
const cargoVersion = readFileSync("Cargo.toml", "utf8").match(
  /\[workspace\.package\][\s\S]*?\nversion = "([^"]+)"/,
)?.[1];

for (const [source, version] of [
  ["package.json", packageVersion],
  ["src-tauri/tauri.conf.json", tauriVersion],
  ["Cargo.toml", cargoVersion],
]) {
  if (version !== expectedVersion) {
    throw new Error(
      `${source} has version ${version}; expected ${expectedVersion} from ${tag}.`,
    );
  }
}
