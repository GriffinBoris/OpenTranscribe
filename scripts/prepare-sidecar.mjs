import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  rmSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const release = process.argv.includes("--release");
const targetArgumentIndex = process.argv.indexOf("--target");
const target =
  targetArgumentIndex >= 0
    ? process.argv[targetArgumentIndex + 1]
    : release
      ? process.env.TAURI_ENV_TARGET_TRIPLE
      : undefined;
const profile = release ? "release" : "debug";
const executableSuffix = process.platform === "win32" ? ".exe" : "";
const buildArguments = [
  "build",
  "--locked",
  "--package",
  "opentranscribe-local-transcriber",
  "--package",
  "opentranscribe-text-normalizer",
];

if (release) {
  const cmake = spawnSync("cmake", ["--version"], { encoding: "utf8" });

  if (cmake.error?.code === "ENOENT") {
    process.stderr.write(
      "CMake 3.20 or newer is required to build the bundled local transcriber.\n",
    );
    process.exit(1);
  }

  if (cmake.status !== 0) {
    process.stderr.write(cmake.stderr);
    process.exit(cmake.status ?? 1);
  }

  const version = cmake.stdout.split("\n")[0].split(" ").at(-1);
  const [major, minor] = version.split(".").map(Number);

  if (major < 3 || (major === 3 && minor < 20)) {
    process.stderr.write(
      `CMake 3.20 or newer is required; found ${version}.\n`,
    );
    process.exit(1);
  }

  buildArguments.push("--release");
}

if (target) {
  buildArguments.push("--target", target);
}

const build = spawnSync("cargo", buildArguments, {
  cwd: projectRoot,
  stdio: "inherit",
});

if (build.status !== 0) {
  process.exit(build.status ?? 1);
}

const compiler = spawnSync("rustc", ["-vV"], {
  cwd: projectRoot,
  encoding: "utf8",
});

if (compiler.status !== 0) {
  process.stderr.write(compiler.stderr);
  process.exit(compiler.status ?? 1);
}

const host =
  target ??
  compiler.stdout
    .split("\n")
    .find((line) => line.startsWith("host: "))
    ?.slice("host: ".length);

if (!host) {
  throw new Error("Unable to determine the Rust host target.");
}

for (const name of ["local-transcriber", "text-normalizer"]) {
  const source = resolve(
    projectRoot,
    "target",
    ...(target ? [target] : []),
    profile,
    `opentranscribe-${name}${executableSuffix}`,
  );
  const destination = resolve(
    projectRoot,
    "apps",
    "desktop",
    "src-tauri",
    "binaries",
    `${name}-${host}${executableSuffix}`,
  );

  if (!existsSync(source)) {
    throw new Error(`Sidecar build did not produce ${source}.`);
  }

  mkdirSync(dirname(destination), { recursive: true });
  rmSync(destination, { force: true });
  copyFileSync(source, destination);

  if (process.platform !== "win32") {
    chmodSync(destination, 0o755);
  }

  process.stdout.write(`Prepared ${destination}\n`);
}
