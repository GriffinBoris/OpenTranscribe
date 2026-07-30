import { execFileSync, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import process from "node:process";

if (process.platform !== "darwin") {
  throw new Error("macOS signing verification must run on macOS.");
}

const signingIdentity = process.env.APPLE_SIGNING_IDENTITY;

if (!signingIdentity) {
  throw new Error(
    "APPLE_SIGNING_IDENTITY must name a valid Developer ID Application identity.",
  );
}

const identities = execFileSync(
  "security",
  ["find-identity", "-v", "-p", "codesigning"],
  { encoding: "utf8" },
);

if (!identities.includes(signingIdentity)) {
  throw new Error(
    `APPLE_SIGNING_IDENTITY was not found in the keychain: ${signingIdentity}`,
  );
}

const appPath = process.argv[2];

if (!appPath) {
  console.log(`Using macOS signing identity: ${signingIdentity}`);
  process.exit(0);
}

if (!existsSync(appPath)) {
  throw new Error(`Signed application bundle was not found: ${appPath}`);
}

execFileSync(
  "codesign",
  ["--verify", "--deep", "--strict", "--verbose=2", appPath],
  { stdio: "inherit" },
);

const inspection = spawnSync("codesign", ["-dvv", appPath], {
  encoding: "utf8",
});
const signatureDetails = `${inspection.stdout}${inspection.stderr}`;

if (inspection.status !== 0) {
  throw new Error(signatureDetails.trim());
}

if (
  signatureDetails.includes("Signature=adhoc") ||
  signatureDetails.includes("TeamIdentifier=not set")
) {
  throw new Error(
    "Application bundle is ad-hoc signed instead of identity signed.",
  );
}

console.log(`Verified signed application bundle: ${appPath}`);
