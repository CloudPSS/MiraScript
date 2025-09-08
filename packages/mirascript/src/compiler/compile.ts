import type { ScriptInput, TranspileOptions } from './types.js';
import { emit } from './emit.js';
import { compileBytecode, compileBytecodeSync } from './compile-bytecode.js';

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

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export function compileSync(script: ScriptInput, options: TranspileOptions): [string | undefined, Uint32Array] {
    const [bytecode, errors] = compileBytecodeSync(script, options);
    if (bytecode == null) {
        return [undefined, errors];
    }
    const generatedCode = emit(script, bytecode, options);
    return [generatedCode, errors];
}
