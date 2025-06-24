// Based on https://github.com/biomejs/biome/blob/a27b8253b2f0d5e5618e9b26eebaaa5da55ed69a/packages/%40biomejs/biome/scripts/update-beta-version.mjs

import * as fs from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SDK_GENERATOR_ROOT = resolve(fileURLToPath(import.meta.url), "../..");
const MANIFEST_PATH = resolve(SDK_GENERATOR_ROOT, "package.json");

const rootManifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf-8"));

if (
  typeof process.env.GITHUB_SHA !== "string" ||
  process.env.GITHUB_SHA === ""
) {
  throw new Error("GITHUB_SHA environment variable is undefined");
}

const version = process.env.INPUT_VERSION;
if (typeof version !== "string" || version === "") {
  throw new Error("INPUT_VERSION environment variable is undefined");
}

rootManifest.version = version;

const content = JSON.stringify(rootManifest);
fs.writeFileSync(MANIFEST_PATH, content);

console.log(version);
