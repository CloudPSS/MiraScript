/* eslint-disable no-console */
import { glob } from 'node:fs/promises';
import path, { posix } from 'node:path';
import { loadPyodide } from 'pyodide';

const wheelDir = path.relative(process.cwd(), path.resolve(process.argv[2] || 'crates/python/dist'));
const wheels = [];
for await (const wheel of glob('*.whl', { cwd: wheelDir })) {
  const url = posix.resolve(wheelDir.replaceAll(path.sep, posix.sep), wheel);
  wheels.push(url);
}

const pyodide = await loadPyodide();
await pyodide.loadPackage(wheels);

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

console.log('Pyodide smoke test output:');
console.log(output);
if (typeof output !== 'string' || !output.includes('def script')) {
  throw new Error('Pyodide smoke test did not produce Python source.');
}
