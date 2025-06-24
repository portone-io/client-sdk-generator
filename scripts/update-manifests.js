// Based on https://github.com/biomejs/biome/blob/a27b8253b2f0d5e5618e9b26eebaaa5da55ed69a/scripts/update-manifests.mjs

import * as fs from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { format } from "node:util";

const REPO_ROOT = resolve(fileURLToPath(import.meta.url), "../..");
const PACKAGES_ROOT = resolve(REPO_ROOT, "packages/@portone");
const PORTONE_CLIENT_SDK_GENERATOR_LIB_PATH = resolve(PACKAGES_ROOT, "client-sdk-generator");
const MANIFEST_PATH = resolve(PORTONE_CLIENT_SDK_GENERATOR_LIB_PATH, "package.json");

const PLATFORMS = ["win32-%s", "darwin-%s", "linux-%s", "linux-%s-musl"];
const ARCHITECTURES = ["x64", "arm64"];

const rootManifest = JSON.parse(
	fs.readFileSync(MANIFEST_PATH).toString("utf-8"),
);

for (const platform of PLATFORMS) {
	for (const arch of ARCHITECTURES) {
		updateOptionalDependencies(platform, arch);
	}
}

function getName(platform, arch, prefix = "client-sdk-generator") {
	return format(`${prefix}-${platform}`, arch);
}

function updateOptionalDependencies(platform, arch) {
	const os = platform.split("-")[0];
	const buildName = getName(platform, arch);
	const packageRoot = resolve(PACKAGES_ROOT, buildName);
	const packageName = `@portone/${buildName}`;

	// Update the package.json manifest
	const { version, license, repository, engines, homepage } = rootManifest;

	const manifest = JSON.stringify(
		{
			name: packageName,
			version,
			license,
			repository: {
				...repository,
				directory: `${repository.directory}/${buildName}`,
			},
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
		},
		null,
		2,
	);

	const manifestPath = resolve(packageRoot, "package.json");
	console.log(`Update manifest ${manifestPath}`);
	fs.writeFileSync(manifestPath, manifest);
}
