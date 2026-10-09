#!/usr/bin/env node
// 同步更新版本号的四处位置（版本号机制见 CONTRIBUTING.md §3 / AGENTS.md §3.2）：
//   package.json、src-tauri/tauri.conf.json、src-tauri/Cargo.toml、src-tauri/core/Cargo.toml
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(__dirname, '..');

const version = process.argv[2]?.replace(/^v/, '');
if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
  console.error('用法: node tools/update-version.mjs <版本号>（例如 1.0.0）');
  process.exit(1);
}

function writeJson(relPath, mutate) {
  const p = path.join(root, relPath);
  const data = JSON.parse(fs.readFileSync(p, 'utf8'));
  mutate(data);
  fs.writeFileSync(p, `${JSON.stringify(data, null, 2)}\n`);
}

writeJson('package.json', (pkg) => {
  pkg.version = version;
});

writeJson(path.join('src-tauri', 'tauri.conf.json'), (conf) => {
  conf.version = version;
});

for (const rel of [path.join('src-tauri', 'Cargo.toml'), path.join('src-tauri', 'core', 'Cargo.toml')]) {
  const p = path.join(root, rel);
  const text = fs.readFileSync(p, 'utf8');
  fs.writeFileSync(p, text.replace(/^version = ".*"$/m, `version = "${version}"`));
}

console.log(`版本号已同步更新至 ${version}`);
