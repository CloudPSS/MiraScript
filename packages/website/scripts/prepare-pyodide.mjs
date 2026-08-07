// @ts-check
import { glob, rm, mkdir, cp } from 'node:fs/promises';
import { basename, relative, resolve, posix, sep } from 'node:path';
import { loadPyodide } from 'pyodide';

const DEFAULT_WHEEL_DIR = resolve(import.meta.dirname, '../../../crates/python/dist');

const websiteDir = resolve(import.meta.dirname, '..');
const outputDir = resolve(websiteDir, 'pyodide.g.assets');
const wheelDir = relative(process.cwd(), resolve(process.argv[2] || DEFAULT_WHEEL_DIR));

const wheels = [];
for await (const wheel of glob('*.whl', { cwd: wheelDir })) {
  if (!wheel.endsWith('-any.whl') && !wheel.endsWith('-pyemscripten_2026_0_wasm32.whl')) {
    continue;
  }
  const url = resolve(wheelDir, wheel);
  wheels.push(url);
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
