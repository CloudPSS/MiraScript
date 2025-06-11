import type { ScriptInput, TranspileOptions } from './types';
import { generate } from './generate';
import { compile } from './compile';

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function transpileCore(
    code: ScriptInput,
    options: TranspileOptions,
): Promise<[Uint8Array, string | undefined, Uint32Array]> {
    const [codeBuf, bytecode, errors] = await compile(code, options);
    if (bytecode == null) {
        return [codeBuf, undefined, errors];
    }
    const generatedCode = generate(bytecode, options);
    return [codeBuf, generatedCode, errors];
}
