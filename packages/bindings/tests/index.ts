import test from 'ava';

import * as bindings from '../dist/index.js';
import * as napi from '../dist/napi.js';
import * as wasm from '../dist/wasm.js';

test('loadModule', async (t) => {
    const module = await bindings.loadModule();
    t.truthy(module);
    t.truthy(module.compileSync);

    const moduleNapi = await napi.loadModule();
    t.truthy(moduleNapi);
    t.truthy(moduleNapi.compile);
    t.truthy(moduleNapi.compileSync);
    t.truthy(moduleNapi.JsCompileResult);

    const moduleWasm = await wasm.loadModule();
    t.truthy(moduleWasm);
    t.truthy(moduleWasm.compileSync);
    t.truthy(moduleWasm.createConfig);
    t.truthy(moduleWasm.formatSync);
    t.truthy(moduleWasm.wasm);
});

test('getModule', async (t) => {
    for (const module of [bindings, napi, wasm]) {
        const loaded = await module.loadModule();
        const gotten = module.getModule();
        t.is(loaded, gotten);
    }
});

test('WASM range formatting preserves template literal content', async (t) => {
    const module = await wasm.loadModule();
    const expression = 'if active { "活跃" } else { "非活跃" }';
    for (const source of [`<p>状态: \${\n  ${expression}\n}</p>`, `<p>状态: \${\n\n  ${expression}\n}</p>`]) {
        const start = source.indexOf(expression);
        const result = module.formatSync(source, { input_mode: 'Template', trivia: true }, undefined, [
            { start, end: start + expression.length },
        ]);
        t.deepEqual(result.edits, []);
    }
});
