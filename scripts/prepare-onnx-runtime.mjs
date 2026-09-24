import { createHash } from "node:crypto";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// Runtime 1.22 supports Intel Mac and the Ubuntu 22.04 baseline. Use API 21.
const checksums = {
  "x86_64-apple-darwin":
    "e0538783248bbb77d2e97556134b8cc47cfc35ac3d762067d51eaf8355c0f48e",
  "x86_64-unknown-linux-gnu":
    "ed1716de95974bf47ab0223ca33734a0b5a5d09a181225d0e8ed62d070aea893",
};

export async function prepareOnnxRuntime(target) {
  const sha256 = checksums[target];
  if (!sha256) return undefined;
  const url = `https://cdn.pyke.io/0/pyke:ort-rs/ms@1.22.0/${target}.tgz`;

  const directory = resolve(
    dirname(fileURLToPath(import.meta.url)),
    `../target/onnxruntime-1.22.0-${target}`,
  );
  mkdirSync(directory, { recursive: true });
  const archive = join(directory, "runtime.tgz");
  const bytes = existsSync(archive)
    ? readFileSync(archive)
    : Buffer.from(await (await fetch(url)).arrayBuffer());
  if (createHash("sha256").update(bytes).digest("hex") !== sha256) {
    throw new Error("ONNX Runtime archive failed its SHA-256 check.");
  }
  writeFileSync(archive, bytes);
  const extract = spawnSync("tar", ["-xzf", archive, "-C", directory], {
    stdio: "inherit",
  });
  if (extract.status !== 0) throw new Error("Could not extract ONNX Runtime.");
  const libraryPath = join(directory, "onnxruntime", "lib");
  // Subsequent workspace tests and Clippy must use the same runtime as the sidecar.
  if (process.env.GITHUB_ENV) {
    appendFileSync(process.env.GITHUB_ENV, `ORT_LIB_PATH=${libraryPath}\n`);
  }
  return libraryPath;
}
