import type { VmScript } from '../vm/index.js';
import type { ScriptInput, TranspileOptions } from './types.js';
import './types.js';
import { emit } from './emit.js';
import { createScript } from './create-script.js';
import { compileFast } from './compile-fast.js';
import { DiagnosticCode, formatDiagnostic, parseDiagnostics } from './diagnostic.js';
import { generateBytecode, generateBytecodeSync, loadModule } from './generate-bytecode.js';
import { compileWorker } from './worker-manager.js';
await loadModule();

export { generateBytecode, generateBytecodeSync };
export type { TranspileOptions, ScriptInput, InputMode } from './types.js';

// 目前编译速度约 2000kB/s
const WORKER_MIN_LEN = typeof Worker != 'function' ? Number.MAX_VALUE : 1024;

/** 报告编译错误 */
function reportDiagnostic(source: ScriptInput, diagnostics: Uint32Array): never {
    const parsed = parseDiagnostics(source, diagnostics);
    const messages = parsed.errors.map(formatDiagnostic);
    throw new Error(`Failed to compile:\n${messages.join('\n')}`);
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export function emitScript(
    source: ScriptInput,
    [code, diagnostics]: [Uint8Array | undefined, Uint32Array],
    options: TranspileOptions,
): VmScript {
    if (!code) {
        reportDiagnostic(source, diagnostics);
    }
    const sourcemaps = options.sourceMap
        ? parseDiagnostics(source, diagnostics, (c) => c === DiagnosticCode.SourceMap).sourcemaps
        : [];
    const target = emit(source, code, sourcemaps, options);
    return createScript(source, target);
}

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
        reportDiagnostic(source, diagnostics);
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
