import type { ScriptInput, TranspileOptions } from './types.js';
import { createScript, type VmScript } from './create-script.js';
import { compileFast } from './compile-fast.js';
import { generateBytecode, generateBytecodeSync } from './generate-bytecode.js';
import { compileWorker } from './worker-manager.js';
import { emitScript, reportDiagnostic } from './emit-script.js';
import './load-module.js';

export * from './types.js';
export type { VmScript };

// 目前编译速度约 2000kB/s
const WORKER_MIN_LEN = typeof Worker != 'function' ? Number.MAX_VALUE : 1024;

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(this: void, source: ScriptInput, options: TranspileOptions = {}): Promise<VmScript> {
    if (options.sourceMap) {
        options.diagnostic_sourcemap = true;
        // https://tc39.es/ecma426/#sec-terms-and-definitions-colun
        options.diagnostic_position_encoding ??= 'Utf16';
    }
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
        reportDiagnostic(source, diagnostics, options.fileName);
    }
    return createScript(source, target);
}
/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export function compileSync(this: void, source: ScriptInput, options: TranspileOptions = {}): VmScript {
    if (typeof source == 'string') {
        const result = compileFast(source, options);
        if (result) return result;
    }
    const bc = generateBytecodeSync(source, options);
    return emitScript(source, bc, options);
}
