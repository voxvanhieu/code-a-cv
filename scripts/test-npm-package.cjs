// Exercise cargo-dist's generated npm launcher with the CI-built native binary.
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const { chmodSync, copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync } = require('node:fs');
const { tmpdir } = require('node:os');
const { join, resolve } = require('node:path');

const packageDir = resolve(process.argv[2]);
const binary = resolve(process.argv[3]);
const pkg = JSON.parse(readFileSync(join(packageDir, 'package.json')));
assert.equal(pkg.name, '@voxvanhieu/code-a-cv');
assert.deepEqual(Object.keys(pkg.bin), ['cac']);
for (const target of ['aarch64-apple-darwin', 'x86_64-apple-darwin',
  'aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc']) {
  assert.ok(pkg.supportedPlatforms[target], `Missing ${target}`);
}
assert.deepEqual(pkg.artifactDownloadUrls,
  [`https://github.com/voxvanhieu/code-a-cv/releases/download/v${pkg.version}`]);

const installed = join(packageDir, 'node_modules', '.bin_real');
mkdirSync(installed, { recursive: true });
const executable = join(installed, process.platform === 'win32' ? 'cac.exe' : 'cac');
copyFileSync(binary, executable);
chmodSync(executable, 0o755);
const cwd = mkdtempSync(join(tmpdir(), 'cac npm smoke '));
function run(args, status = 0) {
  const result = spawnSync(process.execPath, [join(packageDir, pkg.bin.cac), ...args],
    { cwd, encoding: 'utf8' });
  assert.ifError(result.error);
  assert.equal(result.status, status, result.stderr);
  return result.stdout;
}
try {
  assert.equal(run(['--version']).trim(), `cac ${pkg.version}`);
  assert.match(run(['--help']), /Usage:/);
  run(['--not-a-real-option'], 2);
  run(['init']);
  assert.ok(readFileSync(join(cwd, 'cv.md')).length);
  console.log('npm launcher: platform mappings, version, help, exit status, and init passed');
} finally {
  rmSync(cwd, { recursive: true, force: true });
}
