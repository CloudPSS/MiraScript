import './load-module.js';
import type { ScriptInput, TranspileOptions } from './types.js';
import { createScript, type VmScript } from './create-script.js';
import { compileFast } from './compile-fast.js';
import { generateBytecode, generateBytecodeSync } from './generate-bytecode.js';
import { compileWorker } from './worker-manager.js';
import { emitScript, CompileError } from './emit-script.js';
import { normalizeTranspileOptions, WORKER_MIN_LEN } from './options.js';

export * from './types.js';
export type { VmScript };
export { CompileError };

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(this: void, source: ScriptInput, options?: TranspileOptions): Promise<VmScript> {
    options = normalizeTranspileOptions(options);
    if (typeof source == 'string') {
        const result = compileFast(source, options);
        if (result) return result;
    }
    if (source.length < WORKER_MIN_LEN) {
        const bc = await generateBytecode(source, options);
        return emitScript(source, bc, options);
    }
    const [target, diagnostics] = await compileWorker(source, options);
    if (target == null) {
        throw new CompileError(source, diagnostics, options.fileName);
    }
    return createScript(source, options.input_mode ?? 'Script', target);
}
/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export function compileSync(this: void, source: ScriptInput, options?: TranspileOptions): VmScript {
    options = normalizeTranspileOptions(options);
    if (typeof source == 'string') {
        const result = compileFast(source, options);
        if (result) return result;
    }
    const bc = generateBytecodeSync(source, options);
    return emitScript(source, bc, options);
}
