import test from 'ava';

import { loadModule as loadNapi } from '../dist/napi.js';
import { loadModule as loadWasm } from '../dist/wasm.js';

const config = {
    diagnostic_position_encoding: 'Utf16',
    diagnostic_sourcemap: true,
    input_mode: 'Script',
    trivia: true,
} as const;

interface FormatResult {
    readonly diagnostics: Uint32Array;
    readonly edits: ReadonlyArray<{ readonly start: number; readonly end: number; readonly text: string }>;
}

/** 将不同后端的类型化数组结果转换为可直接比较的普通对象。 */
function normalize(result: FormatResult) {
    return {
        diagnostics: [...result.diagnostics],
        edits: result.edits.map((edit) => ({ ...edit })),
    };
}

test('NAPI and WASM format contracts agree for document and UTF-16 ranges', async (t) => {
    const [napi, wasm] = await Promise.all([loadNapi(), loadWasm()]);
    const source = 'let emoji="😀";\nlet x=[1,2,3,4];';
    const options = { tabSize: 2, insertSpaces: true, printWidth: 16 };
    const ranges = [{ start: source.indexOf('let x'), end: source.length }];

    t.deepEqual(
        normalize(napi.formatSync(source, config, options)),
        normalize(wasm.formatSync(source, config, options)),
    );
    t.deepEqual(
        normalize(napi.formatSync(source, config, options, ranges)),
        normalize(wasm.formatSync(source, config, options, ranges)),
    );

    const templateConfig = { ...config, input_mode: 'Template' } as const;
    const template = '<p>Hello</p>';
    const napiTemplate = normalize(napi.formatSync(template, templateConfig, options));
    const wasmTemplate = normalize(wasm.formatSync(template, templateConfig, options));
    t.deepEqual(napiTemplate, wasmTemplate);
    t.deepEqual(napiTemplate.edits, []);
});
