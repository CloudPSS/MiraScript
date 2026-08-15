// @ts-check
import { glob, rm, mkdir, cp, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { basename, relative, resolve, posix, sep } from 'node:path';
import { loadPyodide } from 'pyodide';
import { spawnSync } from 'node:child_process';

const DEFAULT_WHEEL_DIR = resolve(import.meta.dirname, '../../../crates/python/dist');

const websiteDir = resolve(import.meta.dirname, '..');
const outputDir = resolve(websiteDir, 'pyodide.g.assets');
const wheelDir = relative(process.cwd(), resolve(process.argv[2] || DEFAULT_WHEEL_DIR));

/**
 * 查找 wheelDir 中的文件
 * @returns {Promise<string[] | null>} 如果没有找到 mirascript wheel，则返回 null
 */
async function findWheels() {
  let hasMirascript = false;
  const wheels = [];
  for await (const wheel of glob('*.whl', { cwd: wheelDir })) {
    if (!wheel.endsWith('-any.whl') && !wheel.endsWith('-pyemscripten_2026_0_wasm32.whl')) {
      continue;
    }
    if (wheel.startsWith('mirascript-')) {
      hasMirascript = true;
    }
    const url = resolve(wheelDir, wheel);
    wheels.push(url);
  }
  if (!hasMirascript) {
    return null;
  }
  return wheels;
}

/**
 * 从 GitHub Actions artifacts 下载 wheel 文件
 */
async function downloadWheels() {
  const artifactsUrl = 'https://nightly.link/CloudPSS/MiraScript/workflows/ci/main/website-wheels.zip';
  const zipPath = resolve(tmpdir(), 'website-wheels.zip');
  const res = await fetch(artifactsUrl);
  if (!res.ok) {
    throw new Error(`Failed to download wheels from ${artifactsUrl}: ${res.status} ${res.statusText}`);
  }
  await mkdir(wheelDir, { recursive: true });
  await rm(wheelDir, { recursive: true, force: true });
  await mkdir(wheelDir, { recursive: true });
  await writeFile(zipPath, await res.bytes());
  const unzip = spawnSync('python3', ['-m', 'zipfile', '-e', zipPath, wheelDir], { stdio: 'pipe' });
  if (unzip.status !== 0) {
    throw new Error(`Failed to unzip wheels`);
  }
}

let wheels = await findWheels();
if (!wheels) {
  // eslint-disable-next-line no-console
  console.warn(`No wheels found in ${wheelDir}, downloading from GitHub Actions artifacts...`);
  await downloadWheels();
  wheels = await findWheels();
  if (!wheels) {
    throw new Error(`No wheels found in ${wheelDir} after downloading from GitHub Actions artifacts.`);
  }
}

const pyodide = await loadPyodide();
await pyodide.loadPackage(
  wheels.map((w) => {
    const f = relative(process.cwd(), w);
    return posix.resolve(f.replaceAll(sep, posix.sep));
  }),
);

const output = pyodide.runPython(String.raw /* python */ `
import ast
from mirascript import compile

outputs = []
for source, mode in [('return 1 + 2;', 'script'), ('Hello ${1 + 2}', 'template')]:
    script, diagnostics = compile(source, input_mode=mode, filename=f'smoke.{mode}')
    if script is None:
        raise RuntimeError(f'Compilation failed: {diagnostics}')
    outputs.append(ast.unparse(script.ast))
'\n'.join(outputs)
`);

if (typeof output !== 'string' || !output.includes('def script')) {
  throw new Error('Pyodide smoke test did not produce Python source.');
}

await rm(outputDir, { recursive: true, force: true });
await mkdir(outputDir, { recursive: true });
for (const wheel of wheels) {
  await cp(wheel, resolve(outputDir, basename(wheel)));
}
