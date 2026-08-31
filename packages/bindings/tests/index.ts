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
