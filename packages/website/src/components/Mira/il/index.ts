import { ILEmitter } from './emit-il';
import { DiagnosticCode, parseDiagnostics } from '@mirascript/mirascript/subtle';

/** 将 MiraScript 字节码转换为可读 IL。 */
export function emitIL(
    source: string,
    bytecode: Uint8Array<ArrayBuffer>,
    diagnostics: Uint32Array<ArrayBuffer>,
): string {
    if (!bytecode || bytecode.length === 0) {
        return '';
    }
    const { sourcemaps } = parseDiagnostics(source, diagnostics, (code) => code === DiagnosticCode.SourceMap);
    const emitter = new ILEmitter(bytecode, {
        source,
        ranges: sourcemaps,
    });
    return emitter.emit();
}
