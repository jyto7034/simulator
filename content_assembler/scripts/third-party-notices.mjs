import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const lock = JSON.parse(readFileSync(join(root, "package-lock.json"), "utf8"));
const cssSources = new Set(["tailwindcss", "tw-animate-css"]);
const records = [];
const sections = [
  "CONTENT ASSEMBLER — THIRD-PARTY NOTICES",
  "Generated from the installed npm production dependencies and CSS sources.",
  "Tauri host and operating-system libraries are not inventoried here.",
  "\nshadcn/ui — copied UI component sources\n",
  readFileSync(join(root, "licenses/shadcn-ui.LICENSE.txt"), "utf8"),
];

for (const [location, entry] of Object.entries(lock.packages)) {
  if (!location.startsWith("node_modules/")) continue;

  const directory = join(root, location);
  const manifestPath = join(directory, "package.json");
  if (!existsSync(manifestPath)) continue; // Optional packages for other OSes.

  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  if (entry.dev && !cssSources.has(manifest.name)) continue;

  const notices = readdirSync(directory, { withFileTypes: true })
    .filter(
      (file) =>
        file.isFile() &&
        /^(licen[cs]e|copying|notice)([._-].*)?$/i.test(file.name),
    )
    .map((file) => file.name)
    .sort();

  records.push({
    name: manifest.name,
    version: manifest.version,
    license: manifest.license ?? entry.license ?? "UNKNOWN",
    location,
    notices,
  });
}

records.sort(
  (a, b) =>
    a.name.localeCompare(b.name) || a.location.localeCompare(b.location),
);

for (const record of records) {
  sections.push(
    `\n${"=".repeat(72)}\n${record.name}@${record.version}\nLicense: ${typeof record.license === "string" ? record.license : JSON.stringify(record.license)}\n`,
  );
  if (record.name === "elkjs") {
    sections.push(
      "Used under EPL-2.0; no local changes to the library.\n" +
        "Source repository: https://github.com/kieler/elkjs\n" +
        `Original npm package: https://registry.npmjs.org/elkjs/-/elkjs-${record.version}.tgz\n`,
    );
  }
  for (const notice of record.notices) {
    sections.push(
      `--- ${notice} ---\n`,
      readFileSync(join(root, record.location, notice), "utf8"),
    );
  }
  if (!record.notices.length) {
    sections.push(
      "No top-level license/notice file was shipped in this npm package.\n",
    );
  }
}

writeFileSync(
  join(root, "public/third-party-notices.txt"),
  sections.join("\n"),
);
writeFileSync(
  join(root, "licenses/npm-runtime.json"),
  `${JSON.stringify(records, null, 2)}\n`,
);
console.log(
  `Collected notices for ${records.length} installed npm packages and shadcn/ui sources.`,
);

const unknown = records.filter((record) => record.license === "UNKNOWN");
if (unknown.length) {
  console.error(
    `Missing license metadata: ${unknown.map((record) => record.name).join(", ")}`,
  );
  process.exitCode = 1;
}
