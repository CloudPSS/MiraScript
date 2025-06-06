import { type editor, Range, type IRange } from '@private/monaco-editor';
import { DiagnosticCode } from 'mira-wasm';

/** 源代码诊断信息 */
export interface SourceDiagnostic {
    /** 代码 */
    readonly code: DiagnosticCode;
    /** 位置 */
    readonly range: IRange;
    /** 引用 */
    readonly references: readonly SourceReference[];
}
/** 源代码引用信息 */
export interface SourceReference {
    /** 代码 */
    readonly code: DiagnosticCode;
    /** 位置 */
    readonly range: IRange;
    /** 反向引用 */
    readonly diagnostic: SourceDiagnostic;
}
/** 源代码定义信息 */
export interface SourceDefinition {
    /** 符号定义位置，字符串表示全局变量，空范围表示隐式定义的变量（如 `it`） */
    readonly range: IRange | string;
    /** 符号定义 */
    readonly definition?: SourceDiagnostic;
    /** 符号引用 */
    readonly references: readonly SourceDiagnostic[];
}

const compileResult = new Map<string, CompileResult>();
/** 编译结果 */
export class CompileResult {
    /** 获取编译结果 */
    static get(uri: string, version?: number): CompileResult | undefined {
        const data = compileResult.get(uri);
        if (data && version != null) {
            return data.version === version ? data : undefined;
        }
        return data;
    }
    /** 写入编译结果 */
    static set(uri: string, version: number, diagnosticsBuffer: ArrayBuffer, chunkBuffer?: ArrayBuffer): CompileResult {
        const diagnostics = new Uint32Array(diagnosticsBuffer);
        const chunk = chunkBuffer ? new Uint8Array(chunkBuffer) : undefined;
        const result = new CompileResult(version, diagnostics, chunk);
        compileResult.set(uri, result);
        return result;
    }
    constructor(
        /** 代码版本 */
        readonly version: number,
        /** 源代码诊断信息 */
        private readonly diagnostics: Uint32Array,
        /** 代码信息 */
        readonly chunk?: Uint8Array,
    ) {}

    private diagnosticsReady = false;
    private readonly _errors: Array<Writable<SourceDiagnostic>> = [];
    private readonly _warnings: Array<Writable<SourceDiagnostic>> = [];
    private readonly _infos: Array<Writable<SourceDiagnostic>> = [];
    private readonly _hints: Array<Writable<SourceDiagnostic>> = [];
    private readonly _tags: Array<Writable<SourceDiagnostic>> = [];
    /** 源代码诊断信息 */
    get errors(): ReadonlyArray<Writable<SourceDiagnostic>> {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._errors;
    }
    /** 源代码诊断信息 */
    get warnings(): ReadonlyArray<Writable<SourceDiagnostic>> {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._warnings;
    }
    /** 源代码诊断信息 */
    get infos(): ReadonlyArray<Writable<SourceDiagnostic>> {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._infos;
    }
    /** 源代码诊断信息 */
    get hints(): ReadonlyArray<Writable<SourceDiagnostic>> {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._hints;
    }
    /** 源代码诊断信息 */
    get tags(): ReadonlyArray<Writable<SourceDiagnostic>> {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._tags;
    }
    /** 分析诊断信息 */
    private readDiagnostics(): void {
        const diagnostics = [];
        const buf = this.diagnostics;
        const bufLen = buf.length;
        for (let i = 0; i < bufLen; i += 5) {
            const startLineNumber = buf[i]!;
            const startColumn = buf[i + 1]!;
            const endLineNumber = buf[i + 2]!;
            const endColumn = buf[i + 3]!;
            const error = buf[i + 4]! as DiagnosticCode;
            diagnostics.push({
                code: error,
                range: {
                    startLineNumber,
                    startColumn,
                    endLineNumber,
                    endColumn,
                },
            } as Writable<SourceDiagnostic & SourceReference>);
        }
        for (let i = 0; i < diagnostics.length; i++) {
            const diagnostic = diagnostics[i]!;
            const { code } = diagnostic;
            if (code > DiagnosticCode.ErrorStart && code < DiagnosticCode.ErrorEnd) {
                this._errors.push(diagnostic);
            } else if (code > DiagnosticCode.WarningStart && code < DiagnosticCode.WarningEnd) {
                this._warnings.push(diagnostic);
            } else if (code > DiagnosticCode.InfoStart && code < DiagnosticCode.InfoEnd) {
                this._infos.push(diagnostic);
            } else if (code > DiagnosticCode.HintStart && code < DiagnosticCode.HintEnd) {
                this._hints.push(diagnostic);
            } else if (code > DiagnosticCode.TagStart && code < DiagnosticCode.TagEnd) {
                this._tags.push(diagnostic);
            } else {
                continue;
            }
            diagnostic.references = [];
            while (i + 1 < diagnostics.length) {
                const ref = diagnostics[i + 1]!;
                if (ref.code < DiagnosticCode.ReferenceStart || ref.code > DiagnosticCode.ReferenceEnd) {
                    break;
                }
                i++;
                ref.diagnostic = diagnostic;
                (diagnostic.references as SourceReference[]).push(ref);
            }
        }
        this.diagnosticsReady = true;
    }

    private _definitions?: SourceDefinition[];
    /** 获取源代码定义 */
    definitions(model: editor.ITextModel): readonly SourceDefinition[] {
        if (this._definitions) {
            return this._definitions;
        }
        const getText = (range: IRange): string | undefined => {
            if (model.getVersionId() !== this.version) {
                return undefined;
            }
            return model.getValueInRange(range);
        };
        const definitions: Array<Writable<Partial<SourceDefinition>>> = [];
        for (const tag of this.tags) {
            const isGlobal = tag.code === DiagnosticCode.GlobalVariable;
            const declRef = tag.references[0];
            const isRef = isGlobal || declRef != null;
            const defRange = isRef ? declRef?.range : tag.range;
            const globalId = isGlobal ? (getText(tag.range) ?? '') : undefined;
            let def = definitions.find((def) =>
                typeof def.range == 'string' ? def.range === globalId : Range.equalsRange(def.range, defRange),
            );
            if (!def) {
                def = {
                    range: isGlobal ? globalId : defRange,
                    definition: undefined,
                    references: [],
                };
                definitions.push(def);
            }
            if (isRef) {
                (def.references as SourceDiagnostic[]).push(tag);
            } else {
                def.definition = tag;
            }
        }

        this._definitions = definitions as SourceDefinition[];
        return this._definitions;
    }
}
