import type { Writable } from 'type-fest';
import { DiagnosticCode, wasm } from '@mirascript/wasm';
import type { ScriptInput } from './types.js';
import { isSafeInteger } from '../helpers/utils.js';
import { checkModule } from './loader.js';

export { DiagnosticCode };

const diagnosticMessages = new Map<DiagnosticCode, string | undefined>();
/** 获取 {@link DiagnosticCode} 对应的消息 */
export function getDiagnosticMessage(code: DiagnosticCode): string | undefined {
    if (!isSafeInteger(code) || code < 0 || code >= 0xffff) {
        throw new RangeError(`Invalid DiagnosticCode: ${code}`);
    }
    if (diagnosticMessages.has(code)) {
        return diagnosticMessages.get(code);
    }
    checkModule();
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

/** 解析后的诊断信息 */
interface ParsedDiagnostics {
    /** 错误诊断信息 */
    errors: SourceDiagnostic[];
    /** 警告诊断信息 */
    warnings: SourceDiagnostic[];
    /** 信息诊断信息 */
    infos: SourceDiagnostic[];
    /** 提示诊断信息 */
    hints: SourceDiagnostic[];
    /** 标签诊断信息 */
    tags: SourceDiagnostic[];

    /** 引用诊断信息 */
    references: SourceReference[];
    /** 标签引用诊断信息 */
    tagsReferences: SourceReference[];

    /** 代码映射信息 */
    sourcemaps: IRange[];
}

/** 分析诊断信息，{@link diagnostic_position_encoding} 不能设为 `None` */
export function parseDiagnostics(
    source: ScriptInput,
    diagnostics: Uint32Array,
    filter?: (code: DiagnosticCode) => boolean,
): ParsedDiagnostics {
    const parsed = [];
    const bufLen = diagnostics.length;
    for (let i = 0; i < bufLen; i += 5) {
        const code = diagnostics[i + 4]! as DiagnosticCode;
        if (filter && !filter(code)) {
            continue;
        }
        const startLineNumber = diagnostics[i]!;
        const startColumn = diagnostics[i + 1]!;
        const endLineNumber = diagnostics[i + 2]!;
        const endColumn = diagnostics[i + 3]!;
        parsed.push({
            code,
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
    const _sourcemaps: IRange[] = [];
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
        } else if (code === DiagnosticCode.SourceMap) {
            _sourcemaps.push(diagnostic.range);
            continue;
        } else {
            // 非法诊断代码，跳过
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
        sourcemaps: _sourcemaps,
    };
}

/** 生成诊断 range 的字符串 */
function formatRange(range: IRange): string {
    if (range.startLineNumber === range.endLineNumber) {
        if (range.startColumn === range.endColumn) {
            return `${range.startLineNumber}:${range.startColumn}`;
        }
        return `${range.startLineNumber}:${range.startColumn}-${range.endColumn}`;
    }
    return `${range.startLineNumber}:${range.startColumn}-${range.endLineNumber}:${range.endColumn}`;
}

/** 生成诊断消息的字符串 */
export function formatDiagnostic(diagnostic: SourceDiagnostic): string {
    const range = formatRange(diagnostic.range);
    const codeName = DiagnosticCode[diagnostic.code] || `Unknown(${diagnostic.code})`;
    let message = getDiagnosticMessage(diagnostic.code);
    for (const ref of diagnostic.references) {
        const refRange = formatRange(ref.range);
        message += `\n    (${refRange}): ${getDiagnosticMessage(ref.code)}`;
    }
    return `  ${codeName}(${range}): ${message}`;
}
