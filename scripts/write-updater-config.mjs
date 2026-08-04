import { writeFileSync } from "node:fs";
import process from "node:process";

// Tauri validates plugins.updater.pubkey whenever it creates updater
// artifacts, so the committed config alone is not enough: the public key only
// exists as a repository variable. This writes the build-time config from that
// variable and rejects a key Tauri would fail on later, deep inside bundling.
//
// A Tauri public key is base64 of a minisign public key file:
//
//   untrusted comment: minisign public key 1A2B3C4D5E6F7890
//   RWTxxxxxxxx...
//
// The key line itself decodes to 42 bytes: the algorithm "Ed", an 8-byte key
// id, and the 32-byte key.
const KEY_LINE_BYTES = 42;
const SIGNATURE_ALGORITHM = "Ed";

const outputPath = process.argv[2];

if (!outputPath) {
  throw new Error("Usage: write-updater-config.mjs <output-path>");
}

const variable = (process.env.OPENTRANSCRIBE_UPDATER_PUBLIC_KEY ?? "").trim();

if (!variable) {
  fail("OPENTRANSCRIBE_UPDATER_PUBLIC_KEY is empty.");
}

const publicKeyFile = resolvePublicKeyFile(variable);
const keyLine = publicKeyFile
  .split("\n")
  .map((line) => line.trim())
  .filter(Boolean)
  .at(-1);

if (!isKeyLine(keyLine)) {
  fail(
    `OPENTRANSCRIBE_UPDATER_PUBLIC_KEY does not contain a minisign public key. Regenerate it with "npm --prefix apps/desktop run tauri signer generate".`,
  );
}

writeFileSync(
  outputPath,
  `${JSON.stringify(
    {
      bundle: { createUpdaterArtifacts: true },
      plugins: {
        updater: {
          pubkey: Buffer.from(publicKeyFile, "utf8").toString("base64"),
        },
      },
    },
    null,
    2,
  )}\n`,
);

process.stdout.write(`Wrote ${outputPath} with the updater public key.\n`);

// The variable is stored by hand, so accept the three forms it realistically
// takes: the base64 Tauri prints, the raw key file, and a bare key line whose
// comment was dropped on the way into the repository variable.
function resolvePublicKeyFile(value) {
  if (value.includes("untrusted comment")) {
    return value;
  }

  const decoded = decodeBase64(value);

  if (decoded?.includes("untrusted comment")) {
    return decoded;
  }

  if (isKeyLine(value)) {
    return `untrusted comment: minisign public key\n${value}`;
  }

  fail(
    "OPENTRANSCRIBE_UPDATER_PUBLIC_KEY is neither a minisign public key nor its base64 form.",
  );
}

function decodeBase64(value) {
  try {
    return Buffer.from(value, "base64").toString("utf8");
  } catch {
    return undefined;
  }
}

function isKeyLine(value) {
  if (!value) {
    return false;
  }

  const decoded = Buffer.from(value, "base64");

  return (
    decoded.length === KEY_LINE_BYTES &&
    decoded.subarray(0, 2).toString("utf8") === SIGNATURE_ALGORITHM
  );
}

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}
