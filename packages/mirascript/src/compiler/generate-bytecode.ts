import { getModule, loadModule } from '@mirascript/bindings';
import type { CompileOptions, ScriptInput } from './types.js';

/** MiraScript 字节码 */
export type VmBytecodeResult = [code: Uint8Array | undefined, diagnostics: Uint32Array];

/**
 * 生成 MiraScript 字节码
 */
export function generateBytecodeSync(script: ScriptInput, options: CompileOptions): VmBytecodeResult {
    const module = getModule();
    const result = module.compileSync(script, options);
    return [result.chunk, result.diagnostics];
}

/**
 * 生成 MiraScript 字节码
 */
export async function generateBytecode(script: ScriptInput, options: CompileOptions): Promise<VmBytecodeResult> {
    if (options == null) {
        throw new TypeError('options must be provided');
    }
    const module = await loadModule();
    const result = 'compile' in module ? await module.compile(script, options) : module.compileSync(script, options);
    return [result.chunk, result.diagnostics];
}
