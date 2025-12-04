import type { ScriptInput, TranspileOptions } from './types.js';
import type { VmBytecodeResult } from './generate-bytecode.js';
import { emit } from './emit/index.js';
import { createScript, type VmScript } from './create-script.js';
import { DiagnosticCode, formatDiagnostics, parseDiagnostics } from './diagnostic.js';

/** 报告编译错误 */
export function reportDiagnostic(source: ScriptInput, diagnostics: Uint32Array, fileName: string | undefined): never {
    const parsed = parseDiagnostics(source, diagnostics);
    const messages = formatDiagnostics(parsed.errors, source, fileName);
    throw new Error(`Failed to compile:\n${messages.join('\n')}`);
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export function emitScript(
    source: ScriptInput,
    [code, diagnostics]: VmBytecodeResult,
    options: TranspileOptions,
): VmScript {
    if (!code) {
        reportDiagnostic(source, diagnostics, options.fileName);
    }
    const sourcemaps = options.sourceMap
        ? parseDiagnostics(source, diagnostics, (c) => c === DiagnosticCode.SourceMap).sourcemaps
        : [];
    const target = emit(source, code, sourcemaps, options);
    return createScript(source, target);
}
