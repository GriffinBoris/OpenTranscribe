import { execFileSync, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import process from "node:process";

const REQUIRED_ENTITLEMENTS = ["com.apple.security.device.audio-input"];

if (process.platform !== "darwin") {
  throw new Error("macOS signing verification must run on macOS.");
}

const appPath = process.argv[2];

if (!appPath) {
  verifyIdentityIsInstalled();
  process.exit(0);
}

if (!existsSync(appPath)) {
  throw new Error(`Signed application bundle was not found: ${appPath}`);
}

verifyBundleSignature(appPath);
verifyBundleEntitlements(appPath);

console.log(`Verified signed application bundle: ${appPath}`);

// Preflight for a local signed build. CI imports the certificate into a
// temporary keychain that the build action owns, so the bundle checks below
// deliberately do not depend on the identity still being resolvable.
function verifyIdentityIsInstalled() {
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

  console.log(`Using macOS signing identity: ${signingIdentity}`);
}

function verifyBundleSignature(bundlePath) {
  execFileSync(
    "codesign",
    ["--verify", "--deep", "--strict", "--verbose=2", bundlePath],
    { stdio: "inherit" },
  );

  const inspection = spawnSync("codesign", ["-dvv", bundlePath], {
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
}

// The Hardened Runtime that notarization requires denies microphone access
// unless the bundle carries the capture entitlements. That failure is silent
// at build time and only appears when a notarized build tries to record, so
// assert the entitlements survived signing.
function verifyBundleEntitlements(bundlePath) {
  const inspection = spawnSync(
    "codesign",
    ["-d", "--entitlements", "-", "--xml", bundlePath],
    { encoding: "utf8" },
  );

  if (inspection.status !== 0) {
    throw new Error(
      `Unable to read entitlements from ${bundlePath}: ${inspection.stderr.trim()}`,
    );
  }

  const entitlements = `${inspection.stdout}${inspection.stderr}`;
  const missing = REQUIRED_ENTITLEMENTS.filter(
    (entitlement) => !entitlements.includes(entitlement),
  );

  if (missing.length > 0) {
    throw new Error(
      `Signed bundle is missing required entitlements: ${missing.join(", ")}`,
    );
  }
}
