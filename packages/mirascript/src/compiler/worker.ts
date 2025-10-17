import type { ScriptInput, TranspileOptions } from './types.ts';
import { emit } from './emit.ts';
import { compileBytecode } from './compile-bytecode.ts';

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(
    script: ScriptInput,
    options: TranspileOptions,
): Promise<[string | undefined, Uint32Array]> {
    const [bytecode, errors] = await compileBytecode(script, options);
    if (bytecode == null) {
        return [undefined, errors];
    }
    const generatedCode = emit(script, bytecode, options);
    return [generatedCode, errors];
}

addEventListener('message', (ev: MessageEvent<[number, ...Parameters<typeof compile>]>) => {
    const { data } = ev;
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
