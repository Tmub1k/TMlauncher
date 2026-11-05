const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');
const fs = require('node:fs');
const path = require('node:path');

describe('Tier 1 - Feature 29: Project File Structure Reorganization', () => {
  const projectRoot = path.resolve(__dirname, '../..');

  it('Verifies clean project root directory free of loose temporary files', () => {
    const rootFiles = fs.readdirSync(projectRoot);
    const disallowedPrefixes = ['temp_', 'test_out_', 'scratch_', 'debug_log_'];
    for (const f of rootFiles) {
      for (const prefix of disallowedPrefixes) {
        equal(f.startsWith(prefix), false, `Root should not contain temporary file: ${f}`);
      }
    }
  });

  it('Validates presence of standardized code layout directories', () => {
    const requiredDirs = ['src', 'src-tauri', 'tests', 'themes'];
    for (const dir of requiredDirs) {
      const full = path.join(projectRoot, dir);
      ok(fs.existsSync(full), `Required directory ${dir} must exist`);
      ok(fs.statSync(full).isDirectory(), `${dir} must be a directory`);
    }
  });

  it('Ensures .agents/teamwork directory contains strictly agent metadata only', () => {
    const teamworkDir = path.join(projectRoot, '.agents', 'teamwork');
    if (fs.existsSync(teamworkDir)) {
      const forbiddenExtensions = ['.rs', '.ts', '.svelte', '.exe', '.dll'];
      function checkFolder(dir) {
        const entries = fs.readdirSync(dir, { withFileTypes: true });
        for (const entry of entries) {
          if (entry.isDirectory()) {
            checkFolder(path.join(dir, entry.name));
          } else {
            const ext = path.extname(entry.name);
            equal(forbiddenExtensions.includes(ext), false, `No binary or source code in teamwork dir: ${entry.name}`);
          }
        }
      }
      checkFolder(teamworkDir);
    }
  });

  it('Verifies theme directory contains modular JSON descriptor files', () => {
    const themesDir = path.join(projectRoot, 'themes');
    if (fs.existsSync(themesDir)) {
      const themeFiles = fs.readdirSync(themesDir);
      ok(themeFiles.length > 0, 'Themes directory must not be empty');
      const hasJson = themeFiles.some(f => f.endsWith('.json'));
      ok(hasJson, 'Themes directory must contain JSON theme manifests');
    }
  });

  it('Checks package.json and Cargo.toml project configuration integrity', () => {
    const pkgPath = path.join(projectRoot, 'package.json');
    const cargoPath = path.join(projectRoot, 'src-tauri', 'Cargo.toml');
    ok(fs.existsSync(pkgPath), 'package.json must exist');
    ok(fs.existsSync(cargoPath), 'Cargo.toml must exist');

    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
    equal(pkg.name, 'omnilauncher');
  });
});
