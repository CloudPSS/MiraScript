import { checkModule, loadModule, module } from './loader.js';
import type { CompileOptions, ScriptInput } from './types.js';

/**
 * 生成 MiraScript 字节码
 */
export function generateBytecodeSync(
    script: ScriptInput,
    options: CompileOptions,
): [Uint8Array | undefined, Uint32Array] {
    checkModule();
    const result = module.compileSync(script, options);
    return [result.chunk, result.diagnostics];
}

/**
 * 生成 MiraScript 字节码
 */
export async function generateBytecode(
    script: ScriptInput,
    options: CompileOptions,
): Promise<[Uint8Array | undefined, Uint32Array]> {
    if (options == null) {
        throw new TypeError('options must be provided');
    }
    await loadModule();
    const result = 'compile' in module ? await module.compile(script, options) : module.compileSync(script, options);
    return [result.chunk, result.diagnostics];
}
