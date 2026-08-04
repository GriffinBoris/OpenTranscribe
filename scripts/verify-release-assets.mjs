import { readdir } from "node:fs/promises";
import { join } from "node:path";
import process from "node:process";

// Every supported target must contribute its installers before a release is
// published. A partially populated release would send some platforms to a
// download that does not exist.
const requiredArtifacts = [
  {
    target: "aarch64-apple-darwin",
    platform: "macOS on Apple Silicon",
    installers: [".dmg"],
  },
  {
    target: "x86_64-apple-darwin",
    platform: "macOS on Intel",
    installers: [".dmg"],
  },
  {
    target: "x86_64-pc-windows-msvc",
    platform: "Windows x64",
    installers: [".exe", ".msi"],
  },
  {
    target: "x86_64-unknown-linux-gnu",
    platform: "Linux x64",
    installers: [".AppImage", ".deb"],
  },
];

const assetsDirectory = process.argv[2];

if (!assetsDirectory) {
  throw new Error("Usage: verify-release-assets.mjs <assets-directory>");
}

const missing = [];

for (const artifact of requiredArtifacts) {
  const directory = join(assetsDirectory, `release-assets-${artifact.target}`);
  const files = await filesIn(directory);

  for (const installer of artifact.installers) {
    if (!files.some((file) => file.endsWith(installer))) {
      missing.push(`${artifact.platform} (${artifact.target}): ${installer}`);
    }
  }
}

if (missing.length > 0) {
  throw new Error(
    `Missing release installers:\n${missing.map((entry) => `  - ${entry}`).join("\n")}`,
  );
}

process.stdout.write(
  `Verified installers for ${requiredArtifacts.length} targets.\n`,
);

async function filesIn(directory) {
  let entries;

  try {
    entries = await readdir(directory, { withFileTypes: true });
  } catch {
    return [];
  }

  const files = await Promise.all(
    entries.map(async (entry) => {
      const path = join(directory, entry.name);
      return entry.isDirectory() ? filesIn(path) : [path];
    }),
  );

  return files.flat();
}
