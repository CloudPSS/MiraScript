import { readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { loadPyodide } from 'pyodide';

const wheelDir = resolve(process.argv[2] || 'crates/python/dist');
const files = readdirSync(wheelDir, { recursive: true }).map(String);
const findWheel = (pattern) => {
  const file = files.find((value) => pattern.test(value.replaceAll('\\', '/')));
  if (!file) throw new Error(`Missing wheel matching ${pattern} under ${wheelDir}`);
  return resolve(wheelDir, file);
};

const mirascriptWheel = findWheel(/mirascript-.*-(?:pyemscripten|pyodide)_.*_wasm32\.whl$/u);
const typingExtensionsWheel = findWheel(/typing_extensions-.*-py3-none-any\.whl$/u);
const pyodide = await loadPyodide();
await pyodide.loadPackage(typingExtensionsWheel);
await pyodide.loadPackage(mirascriptWheel);

const output = pyodide.runPython(String.raw`
import ast
from mirascript import compile

outputs = []
for source, mode in [('return 1 + 2', 'script'), ('Hello {{ 1 + 2 }}', 'template')]:
    script, diagnostics = compile(source, input_mode=mode, filename=f'smoke.{mode}')
    if script is None:
        raise RuntimeError(f'Compilation failed: {diagnostics}')
    outputs.append(ast.unparse(script.ast))
'\n'.join(outputs)
`);

if (typeof output !== 'string' || !output.includes('def script')) {
  throw new Error('Pyodide smoke test did not produce Python source.');
}
