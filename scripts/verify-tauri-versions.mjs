import { readFileSync } from "node:fs";

const frontend = JSON.parse(
  readFileSync("apps/desktop/package-lock.json", "utf8"),
).packages;
const nativePackages = readFileSync("Cargo.lock", "utf8").split("[[package]]");

for (const name of Object.keys(frontend[""].dependencies)) {
  const nativeName =
    name === "@tauri-apps/api"
      ? "tauri"
      : name.startsWith("@tauri-apps/plugin-")
        ? name.replace("@tauri-apps/", "tauri-")
        : undefined;
  if (!nativeName) continue;

  const matches = nativePackages.filter(
    (entry) => entry.match(/^name = "([^"]+)"/m)?.[1] === nativeName,
  );
  if (matches.length !== 1) {
    throw new Error(`Expected one locked Rust package for ${nativeName}.`);
  }

  const nativeVersion = matches[0].match(/^version = "([^"]+)"/m)[1];
  const frontendVersion = frontend[`node_modules/${name}`].version;
  if (
    nativeVersion.split(".").slice(0, 2).join(".") !==
    frontendVersion.split(".").slice(0, 2).join(".")
  ) {
    throw new Error(
      `${name} ${frontendVersion} and ${nativeName} ${nativeVersion} must share a major/minor version.`,
    );
  }
}
