import { spawnSync } from "node:child_process";

const [maximumVersion, ...binaries] = process.argv.slice(2);

if (!maximumVersion || binaries.length === 0) {
  throw new Error(
    "Usage: node scripts/verify-linux-glibc.mjs <maximum-version> <binary> [...binary]",
  );
}

function compareVersions(left, right) {
  const [leftMajor, leftMinor] = left.split(".").map(Number);
  const [rightMajor, rightMinor] = right.split(".").map(Number);

  return leftMajor - rightMajor || leftMinor - rightMinor;
}

for (const binary of binaries) {
  const result = spawnSync("readelf", ["--version-info", binary], {
    encoding: "utf8",
  });

  if (result.status !== 0) {
    throw new Error(result.stderr.trim());
  }

  const versions = [...result.stdout.matchAll(/GLIBC_(\d+\.\d+)/g)].map(
    ([, version]) => version,
  );
  const requiredVersion = versions.sort(compareVersions).at(-1);

  if (!requiredVersion) {
    throw new Error(`${binary} does not reference a GLIBC version.`);
  }

  if (compareVersions(requiredVersion, maximumVersion) > 0) {
    throw new Error(
      `${binary} requires GLIBC_${requiredVersion}; maximum supported is GLIBC_${maximumVersion}.`,
    );
  }

  process.stdout.write(`${binary}: GLIBC_${requiredVersion}\n`);
}
