/* eslint-disable no-console */
import { createHash } from 'node:crypto';
import { cpSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { basename, dirname, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const websiteDir = resolve(import.meta.dirname, '..');
const repositoryDir = resolve(websiteDir, '../..');
const outputDir = resolve(websiteDir, 'static/pyodide.g.assets');
const wheelDir = resolve(process.env.MIRASCRIPT_PYODIDE_WHEELS || resolve(repositoryDir, 'crates/python/dist'));
const pyodideDir = dirname(fileURLToPath(import.meta.resolve('pyodide/package.json')));

const runtimeFiles = ['pyodide.mjs', 'pyodide.asm.mjs', 'pyodide.asm.wasm', 'pyodide-lock.json', 'python_stdlib.zip'];

/**
 * 递归查找 wheel。
 * @param {RegExp} pattern 文件名规则
 * @returns {string | undefined} wheel 路径
 */
function findWheel(pattern) {
  const pending = [wheelDir];
  while (pending.length > 0) {
    const directory = pending.pop();
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = resolve(directory, entry.name);
      if (entry.isDirectory()) {
        pending.push(path);
      } else if (pattern.test(entry.name)) {
        return path;
      }
    }
  }
  return undefined;
}

let mirascriptWheel;
let typingExtensionsWheel;
try {
  mirascriptWheel = findWheel(/^mirascript-.*-(?:pyemscripten|pyodide)_.*_wasm32\.whl$/u);
  typingExtensionsWheel = findWheel(/^typing_extensions-.*-py3-none-any\.whl$/u);
} catch (error) {
  if (!(error instanceof Error) || !('code' in error) || error.code !== 'ENOENT') throw error;
}

if (!mirascriptWheel || !typingExtensionsWheel) {
  throw new Error(
    `Missing Pyodide wheels under ${wheelDir}. Run \`pyodide build\` in crates/python and download a ` +
      '`typing_extensions` wheel there, or set MIRASCRIPT_PYODIDE_WHEELS.',
  );
}

rmSync(outputDir, { recursive: true, force: true });
mkdirSync(outputDir, { recursive: true });
for (const file of runtimeFiles) {
  cpSync(resolve(pyodideDir, file), resolve(outputDir, file));
}

const mirascriptWheelName = basename(mirascriptWheel);
const typingExtensionsWheelName = basename(typingExtensionsWheel);
const wheelHash = createHash('sha256')
  .update(readFileSync(mirascriptWheel))
  .update(readFileSync(typingExtensionsWheel))
  .digest('hex')
  .slice(0, 16);
const wheelOutputDir = resolve(outputDir, 'wheels', wheelHash);
mkdirSync(wheelOutputDir, { recursive: true });
cpSync(mirascriptWheel, resolve(wheelOutputDir, mirascriptWheelName));
cpSync(typingExtensionsWheel, resolve(wheelOutputDir, typingExtensionsWheelName));
const wheels = {
  mirascript: `wheels/${wheelHash}/${mirascriptWheelName}`,
  typingExtensions: `wheels/${wheelHash}/${typingExtensionsWheelName}`,
};
writeFileSync(resolve(outputDir, 'manifest.json'), `${JSON.stringify({ wheels }, undefined, 2)}\n`);

console.log(`Prepared Pyodide runtime and wheels in ${relative(repositoryDir, outputDir)}`);
