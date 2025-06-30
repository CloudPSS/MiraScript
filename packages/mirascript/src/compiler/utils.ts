import { DiagnosticCode, wasm } from '@mirascript/wasm';

export { DiagnosticCode };
const diagnosticMessages = new Map<DiagnosticCode, string | undefined>();
/** 获取 {@link DiagnosticCode} 对应的消息 */
export function getDiagnosticMessage(code: DiagnosticCode): string | undefined {
    if (!Number.isSafeInteger(code) || code < 0 || code >= 0xffff) {
        throw new RangeError(`Invalid DiagnosticCode: ${code}`);
    }
    if (diagnosticMessages.has(code)) {
        return diagnosticMessages.get(code);
    }
    const msg = wasm.get_diagnostic_message(code);
    diagnosticMessages.set(code, msg);
    return msg;
}

/**
 * A range in the editor. This interface is suitable for serialization.
 */
export interface IRange {
    /**
     * Line number on which the range starts (starts at 1).
     */
    readonly startLineNumber: number;
    /**
     * Column on which the range starts in line `startLineNumber` (starts at 1).
     */
    readonly startColumn: number;
    /**
     * Line number on which the range ends.
     */
    readonly endLineNumber: number;
    /**
     * Column on which the range ends in line `endLineNumber`.
     */
    readonly endColumn: number;
}

/** 源代码诊断信息 */
interface SourceDiagnosticBase<T extends DiagnosticCode = DiagnosticCode> {
    /** 代码 */
    readonly code: T;
    /** 位置 */
    readonly range: IRange;
}

/** 源代码诊断信息 */
export interface SourceDiagnostic<T extends DiagnosticCode = DiagnosticCode> extends SourceDiagnosticBase<T> {
    /** 引用 */
    readonly references: ReadonlyArray<SourceReference<T>>;
}
/** 源代码引用信息 */
export interface SourceReference<T extends DiagnosticCode = DiagnosticCode> extends SourceDiagnosticBase<T> {
    /** 反向引用 */
    readonly diagnostic: SourceDiagnostic<T>;
}

/** 分析诊断信息 */
export function parseDiagnostics(
    source: string,
    diagnostics: Uint32Array,
): {
    errors: SourceDiagnostic[];
    warnings: SourceDiagnostic[];
    infos: SourceDiagnostic[];
    hints: SourceDiagnostic[];
    tags: SourceDiagnostic[];

    references: SourceReference[];
    tagsReferences: SourceReference[];
} {
    const parsed = [];
    const bufLen = diagnostics.length;
    for (let i = 0; i < bufLen; i += 5) {
        const startLineNumber = diagnostics[i]!;
        const startColumn = diagnostics[i + 1]!;
        const endLineNumber = diagnostics[i + 2]!;
        const endColumn = diagnostics[i + 3]!;
        const error = diagnostics[i + 4]! as DiagnosticCode;
        parsed.push({
            code: error,
            range: {
                startLineNumber,
                startColumn,
                endLineNumber,
                endColumn,
            },
        } as Writable<SourceDiagnostic & SourceReference>);
    }

    const _errors: SourceDiagnostic[] = [];
    const _warnings: SourceDiagnostic[] = [];
    const _infos: SourceDiagnostic[] = [];
    const _hints: SourceDiagnostic[] = [];
    const _tags: SourceDiagnostic[] = [];
    const _references: SourceReference[] = [];
    const _tagsReferences: SourceReference[] = [];
    for (let i = 0; i < parsed.length; i++) {
        const diagnostic = parsed[i]!;
        const { code } = diagnostic;
        if (code > DiagnosticCode.ErrorStart && code < DiagnosticCode.ErrorEnd) {
            _errors.push(diagnostic);
        } else if (code > DiagnosticCode.WarningStart && code < DiagnosticCode.WarningEnd) {
            _warnings.push(diagnostic);
        } else if (code > DiagnosticCode.InfoStart && code < DiagnosticCode.InfoEnd) {
            _infos.push(diagnostic);
        } else if (code > DiagnosticCode.HintStart && code < DiagnosticCode.HintEnd) {
            _hints.push(diagnostic);
        } else if (code > DiagnosticCode.TagStart && code < DiagnosticCode.TagEnd) {
            _tags.push(diagnostic);
        } else {
            continue;
        }
        diagnostic.references = [];
        while (i + 1 < parsed.length) {
            const ref = parsed[i + 1]!;
            let isRef = false;
            if (ref.code > DiagnosticCode.TagRefStart && ref.code < DiagnosticCode.TagRefEnd) {
                isRef = true;
                _tagsReferences.push(ref);
            }
            if (ref.code > DiagnosticCode.ReferenceStart && ref.code < DiagnosticCode.ReferenceEnd) {
                isRef = true;
                _references.push(ref);
            }
            if (!isRef) {
                break;
            }
            i++;
            ref.diagnostic = diagnostic;
            (diagnostic.references as SourceReference[]).push(ref);
        }
    }
    return {
        errors: _errors,
        warnings: _warnings,
        infos: _infos,
        hints: _hints,
        tags: _tags,
        references: _references,
        tagsReferences: _tagsReferences,
    };
}
