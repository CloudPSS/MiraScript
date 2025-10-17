import type { VmScript } from '../vm/index.js';
import type { ScriptInput, TranspileOptions } from './types.js';
import './types.js';
import { emit } from './emit.js';
import { createScript } from './create-script.js';
import { compileFast } from './compile-fast.js';
import { formatDiagnostic, parseDiagnostics } from './diagnostic.js';
import { compileBytecode, compileBytecodeSync, loadModule } from './compile-bytecode.js';
import { compileWorker } from './worker-manager.js';
await loadModule();

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
function emitImpl(
    source: ScriptInput,
    [code, diagnostics]: [Uint8Array | undefined, Uint32Array],
    options: TranspileOptions,
): VmScript {
    if (!code) {
        reportDiagnostic(source, diagnostics);
    }
    const target = emit(source, code, options);
    return createScript(source, target);
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(this: void, source: ScriptInput, options: TranspileOptions = {}): Promise<VmScript> {
    if (typeof source == 'string') {
        const result = compileFast(source, options);
        if (result) return result;
    }
    if (source.length < WORKER_MIN_LEN) {
        const bc = await compileBytecode(source, options);
        return emitImpl(source, bc, options);
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
    const bc = compileBytecodeSync(source, options);
    return emitImpl(source, bc, options);
}
