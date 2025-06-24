// Based on https://github.com/biomejs/biome/blob/503e8ef886e75b6091e9af190cd245cec05ea44e/packages/%40biomejs/biome/scripts/generate-packages.mjs

import * as fs from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { format } from "node:util";

const CLI_ROOT = resolve(fileURLToPath(import.meta.url), "../..");
const PACKAGES_ROOT = resolve(CLI_ROOT, "..");
const REPO_ROOT = resolve(PACKAGES_ROOT, "../..");
const MANIFEST_PATH = resolve(CLI_ROOT, "package.json");

const rootManifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf-8"));

function getName(platform, arch, prefix = "client-sdk-generator") {
  return format(`${prefix}-${platform}`, arch);
}

function copyBinaryToNativePackage(platform, arch) {
  const os = platform.split("-")[0];
  const buildName = getName(platform, arch);
  const packageRoot = resolve(PACKAGES_ROOT, buildName);
  const packageName = `@portone/${buildName}`;

  // Update the package.json manifest
  const { version, license, repository, engines, homepage, type, publishConfig } = rootManifest;

  const manifest = JSON.stringify(
    {
      name: packageName,
      version,
      type,
      license,
      repository,
      engines,
      homepage,
      os: [os],
      cpu: [arch],
      libc:
        os === "linux"
          ? packageName.endsWith("musl")
            ? ["musl"]
            : ["glibc"]
          : undefined,
      publishConfig,
    },
    null,
    2,
  );

  const manifestPath = resolve(packageRoot, "package.json");
  console.info(`Update manifest ${manifestPath}`);
  fs.writeFileSync(manifestPath, manifest);

  // Copy the Client SDK Generator binary
  const ext = os === "win32" ? ".exe" : "";
  const binarySource = resolve(
    REPO_ROOT,
    `${getName(platform, arch, "portone-client-sdk-generator")}${ext}`,
  );
  const binaryTarget = resolve(packageRoot, `portone-client-sdk-generator${ext}`);

  if (!fs.existsSync(binarySource)) {
    console.error(
      `Source for binary for ${buildName} not found at: ${binarySource}`,
    );
    process.exit(1);
  }

  console.info(`Copy binary ${binaryTarget}`);
  fs.copyFileSync(binarySource, binaryTarget);
  fs.chmodSync(binaryTarget, 0o755);
}

const PLATFORMS = ["win32-%s", "darwin-%s", "linux-%s", "linux-%s-musl"];
const ARCHITECTURES = ["x64", "arm64"];

for (const platform of PLATFORMS) {
  for (const arch of ARCHITECTURES) {
    copyBinaryToNativePackage(platform, arch);
  }
}
