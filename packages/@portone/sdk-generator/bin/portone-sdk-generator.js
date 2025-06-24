// Based on https://github.com/biomejs/biome/blob/ca81e98ffd9aa648db6b746e26d0dfcce3d3c8c1/packages/%40biomejs/biome/bin/biome
import { execSync, spawnSync } from "node:child_process";
import { arch, env, platform } from "node:process";

function isMusl() {
  let stderr;
  try {
    stderr = execSync("ldd --version", {
      stdio: ["pipe", "pipe", "pipe"],
    });
  } catch (err) {
    stderr = err.stderr;
  }
  if (stderr.indexOf("musl") > -1) {
    return true;
  }
  return false;
}

const PLATFORMS = {
  win32: {
    x64: "@portone/sdk-generator-win32-x64/portone-sdk-generator.exe",
    arm64: "@portone/sdk-generator-win32-arm64/portone-sdk-generator.exe",
  },
  darwin: {
    x64: "@portone/sdk-generator-darwin-x64/portone-sdk-generator",
    arm64: "@portone/sdk-generator-darwin-arm64/portone-sdk-generator",
  },
  linux: {
    x64: "@portone/sdk-generator-linux-x64/portone-sdk-generator",
    arm64: "@portone/sdk-generator-linux-arm64/portone-sdk-generator",
  },
  "linux-musl": {
    x64: "@portone/sdk-generator-linux-x64-musl/portone-sdk-generator",
    arm64: "@portone/sdk-generator-linux-arm64-musl/portone-sdk-generator",
  },
};

const binPath =
  platform === "linux" && isMusl()
    ? PLATFORMS?.["linux-musl"]?.[arch]
    : PLATFORMS?.[platform]?.[arch];

if (binPath) {
  const result = spawnSync(
    import.meta.resolve(binPath),
    process.argv.slice(2),
    {
      shell: false,
      stdio: "inherit",
      env: {
        ...env,
      },
    },
  );

  if (result.error) {
    throw result.error;
  }

  process.exitCode = result.status ?? 0;
} else {
  console.error(
    "The PortOne SDK Generator package doesn't ship with prebuilt binaries for your platform yet. ",
  );
  process.exitCode = 1;
}
