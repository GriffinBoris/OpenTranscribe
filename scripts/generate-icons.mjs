import { execFile } from "node:child_process";
import { copyFile, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const run = promisify(execFile);
const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tauriCli = join(
  repositoryRoot,
  "apps",
  "desktop",
  "node_modules",
  "@tauri-apps",
  "cli",
  "tauri.js",
);
const appIcon = join(
  repositoryRoot,
  "apps",
  "desktop",
  "src",
  "assets",
  "opentranscribe-icon.svg",
);
const trayIcon = join(
  repositoryRoot,
  "apps",
  "desktop",
  "src",
  "assets",
  "opentranscribe-tray.svg",
);
const nativeIcons = join(
  repositoryRoot,
  "apps",
  "desktop",
  "src-tauri",
  "icons",
);
const trayOutput = await mkdtemp(join(tmpdir(), "opentranscribe-tray-"));

try {
  await run(
    process.execPath,
    [tauriCli, "icon", appIcon, "--output", nativeIcons],
    { cwd: join(repositoryRoot, "apps", "desktop") },
  );
  await run(
    process.execPath,
    [tauriCli, "icon", trayIcon, "--output", trayOutput, "--png", "32"],
    { cwd: join(repositoryRoot, "apps", "desktop") },
  );
  await copyFile(
    join(trayOutput, "32x32.png"),
    join(nativeIcons, "tray-icon.png"),
  );
} finally {
  await rm(trayOutput, { recursive: true });
}
