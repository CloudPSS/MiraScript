import type { ScriptInput, TranspileOptions } from './types.js';
import { emit } from './emit/index.js';
import { generateBytecode } from './generate-bytecode.js';
import { DiagnosticCode, parseDiagnostics } from './diagnostic.js';

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(
    script: ScriptInput,
    options: TranspileOptions,
): Promise<[string | undefined, Uint32Array]> {
    const [bytecode, errors] = await generateBytecode(script, options);
    if (bytecode == null) {
        return [undefined, errors];
    }
    const sourcemaps = options.sourceMap
        ? parseDiagnostics(script, errors, (c) => c === DiagnosticCode.SourceMap).sourcemaps
        : [];
    const generatedCode = emit(script, bytecode, sourcemaps, options);
    return [generatedCode, errors];
}

addEventListener('message', (ev) => {
    const data = ev.data as [number, ...Parameters<typeof compile>];
    if (!Array.isArray(data)) return;
    const [seq, ...args] = data;
    if (typeof seq != 'number' || !args.length) return;
    void compile(...args)
        .then(([script, errors]) => {
            postMessage([seq, script, errors], { transfer: [errors.buffer] });
        })
        .catch((error) => {
            postMessage([seq, error instanceof Error ? error : new Error(String(error))]);
        });
});

compile('{}', {}).then(
    () => postMessage('ready'),
    (ex) => postMessage(ex),
);
