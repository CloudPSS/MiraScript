import type { TranspileOptions } from './options';
import { generate } from './generate';
import { compile } from './compile';

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function transpileCore(
    code: string,
    options: TranspileOptions,
): Promise<[string | undefined, Uint32Array]> {
    const [bytecode, errors] = await compile(code, options);
    if (bytecode == null) {
        return [undefined, errors];
    }
    const generatedCode = generate(bytecode, options);
    return [generatedCode, errors];
}
