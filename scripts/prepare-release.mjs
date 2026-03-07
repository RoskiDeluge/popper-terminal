import { readFileSync, writeFileSync } from 'node:fs';

const version = process.argv[2];

if (!version || !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
  console.error('Usage: npm run release:prepare -- <semver>');
  process.exit(1);
}

const updates = [
  {
    path: 'package.json',
    apply(content) {
      const data = JSON.parse(content);
      if (data.version === version) {
        return content;
      }
      data.version = version;
      return `${JSON.stringify(data, null, 2)}\n`;
    },
  },
  {
    path: 'package-lock.json',
    apply(content) {
      const data = JSON.parse(content);
      const packageVersion = data.packages?.['']?.version;
      if (data.version === version && packageVersion === version) {
        return content;
      }
      data.version = version;
      if (data.packages?.['']) {
        data.packages[''].version = version;
      }
      return `${JSON.stringify(data, null, 2)}\n`;
    },
  },
  {
    path: 'src-tauri/Cargo.toml',
    apply(content) {
      return content.replace(
        /(\[package\][\s\S]*?^version\s*=\s*")([^"]+)(")/m,
        `$1${version}$3`,
      );
    },
  },
  {
    path: 'src-tauri/tauri.conf.json',
    apply(content) {
      const data = JSON.parse(content);
      if (data.version === version) {
        return content;
      }
      data.version = version;
      return `${JSON.stringify(data, null, 2)}\n`;
    },
  },
];

for (const update of updates) {
  const original = readFileSync(update.path, 'utf8');
  const next = update.apply(original);

  if (next === original) {
    console.log(`No change: ${update.path}`);
    continue;
  }

  writeFileSync(update.path, next);
  console.log(`Updated ${update.path} -> ${version}`);
}

console.log('');
console.log(`Next steps:`);
console.log(`  npm run build`);
console.log(`  cargo build --manifest-path src-tauri/Cargo.toml`);
console.log(`  git add . && git commit -m "Prepare v${version}"`);
console.log(`  git tag v${version} && git push origin main && git push origin v${version}`);
