import { readdir, readFile, stat, writeFile } from "node:fs/promises";
import { basename, join } from "node:path";

const REPOSITORY = "GriffinBoris/OpenTranscribe";
const RELEASE_ASSET_ROOT = `https://github.com/${REPOSITORY}/releases/download`;

const options = readOptions(process.argv.slice(2));
const assetsDirectory = requiredOption(options, "assets-directory");
const outputPath = requiredOption(options, "output");
const releaseTag = requiredOption(options, "release-tag");
const version = requiredOption(options, "version");

const platformArtifacts = [
  {
    platform: "darwin-aarch64",
    artifactDirectory: "release-assets-aarch64-apple-darwin",
    matches: (path) => path.endsWith(".app.tar.gz"),
  },
  {
    platform: "darwin-x86_64",
    artifactDirectory: "release-assets-x86_64-apple-darwin",
    matches: (path) => path.endsWith(".app.tar.gz"),
  },
  {
    platform: "windows-x86_64",
    artifactDirectory: "release-assets-x86_64-pc-windows-msvc",
    matches: (path) => path.endsWith(".exe"),
  },
  {
    platform: "linux-x86_64",
    artifactDirectory: "release-assets-x86_64-unknown-linux-gnu",
    matches: (path) => path.endsWith(".AppImage"),
  },
];

const platforms = {};

for (const artifact of platformArtifacts) {
  const directory = join(assetsDirectory, artifact.artifactDirectory);
  const files = await filesIn(directory);
  const updatePath = files.find(artifact.matches);

  if (!updatePath) {
    throw new Error(
      `Missing ${artifact.platform} updater artifact in ${directory}.`,
    );
  }

  const signaturePath = `${updatePath}.sig`;

  try {
    await stat(signaturePath);
  } catch {
    throw new Error(`Missing signature for ${basename(updatePath)}.`);
  }

  const signature = (await readFile(signaturePath, "utf8")).trim();

  if (!signature) {
    throw new Error(`Empty signature for ${basename(updatePath)}.`);
  }

  platforms[artifact.platform] = {
    signature,
    url: `${RELEASE_ASSET_ROOT}/${releaseTag}/${encodeURIComponent(basename(updatePath))}`,
  };
}

await writeFile(
  outputPath,
  `${JSON.stringify(
    {
      version,
      notes: `OpenTranscribe ${version}`,
      pub_date: new Date().toISOString(),
      platforms,
    },
    null,
    2,
  )}\n`,
);

function readOptions(arguments_) {
  const values = new Map();

  for (let index = 0; index < arguments_.length; index += 2) {
    const name = arguments_[index];
    const value = arguments_[index + 1];

    if (!name?.startsWith("--") || !value) {
      throw new Error(
        `Expected a value after ${name ?? "the final argument"}.`,
      );
    }

    values.set(name.slice(2), value);
  }

  return values;
}

function requiredOption(options_, name) {
  const value = options_.get(name);

  if (!value) {
    throw new Error(`Missing --${name}.`);
  }

  return value;
}

async function filesIn(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const path = join(directory, entry.name);
      return entry.isDirectory() ? filesIn(path) : [path];
    }),
  );

  return files.flat();
}
