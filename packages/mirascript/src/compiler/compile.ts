import type { ParseMode, ScriptInput, TranspileOptions } from './types';
import { emit } from './emit';
import { compileBytecode } from './compile-bytecode';

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(
    code: ScriptInput,
    mode: ParseMode,
    options: TranspileOptions,
): Promise<[Uint8Array, string | undefined, Uint32Array]> {
    const [codeBuf, bytecode, errors] = await compileBytecode(code, mode, options);
    if (bytecode == null) {
        return [codeBuf, undefined, errors];
    }
    const generatedCode = emit(bytecode, options);
    return [codeBuf, generatedCode, errors];
}
