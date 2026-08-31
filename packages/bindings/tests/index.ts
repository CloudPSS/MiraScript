import test from 'ava';

import { loadModule } from '../dist/index.js';
import { loadModule as loadNapi } from '../dist/napi.js';
import { loadModule as loadWasm } from '../dist/wasm.js';

test('loadModule', async (t) => {
    const module = await loadModule();
    t.truthy(module);
    t.truthy(module.compileSync);

    const moduleNapi = await loadNapi();
    t.truthy(moduleNapi);
    t.truthy(moduleNapi.compile);
    t.truthy(moduleNapi.compileSync);
    t.truthy(moduleNapi.JsCompileResult);

    const moduleWasm = await loadWasm();
    t.truthy(moduleWasm);
    t.truthy(moduleWasm.compileSync);
    t.truthy(moduleWasm.createConfig);
    t.truthy(moduleWasm.formatSync);
    t.truthy(moduleWasm.wasm);
});

test('WASM range formatting preserves template literal content', async (t) => {
    const module = await loadWasm();
    const expression = 'if active { "活跃" } else { "非活跃" }';
    for (const source of [`<p>状态: \${\n  ${expression}\n}</p>`, `<p>状态: \${\n\n  ${expression}\n}</p>`]) {
        const start = source.indexOf(expression);
        const result = module.formatSync(source, { input_mode: 'Template', trivia: true }, undefined, [
            { start, end: start + expression.length },
        ]);
        t.deepEqual(result.edits, []);
    }
});
