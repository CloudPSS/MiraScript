import type { ScriptInput, TranspileOptions } from './types';
import { emit } from './emit';
import { compileBytecode } from './compile-bytecode';

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(
    script: ScriptInput,
    options: TranspileOptions,
): Promise<[Uint8Array, string | undefined, Uint32Array]> {
    const [codeBuf, bytecode, errors] = await compileBytecode(script, options);
    if (bytecode == null) {
        return [codeBuf, undefined, errors];
    }
    const generatedCode = emit(script, bytecode, options);
    return [codeBuf, generatedCode, errors];
}
