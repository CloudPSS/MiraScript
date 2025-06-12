import { type editor, Range, type IRange, type IPosition } from 'monaco-editor';
import { DiagnosticCode } from '@mirascript/wasm';
import { strictContainsPosition } from './utils.js';

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

/** 源代码定义信息 */
interface SourceDefinitionBase<R extends DiagnosticCode = DiagnosticCode> {
    /** 符号引用 */
    readonly references: ReadonlyArray<SourceDiagnostic<R> | SourceReference<R>>;
}
/** 局部函数类型 */
export type LocalFunctionType = (typeof LocalFunctionType)[number];
export const LocalFunctionType = [DiagnosticCode.LocalFunction] as const;

/** 局部变量类型 */
export type LocalVariableType = (typeof LocalVariableType)[number];
export const LocalVariableType = [DiagnosticCode.LocalMutable, DiagnosticCode.LocalImmutable] as const;
/** 显式参数类型 */
export type ParameterExplicitType = (typeof ParameterExplicitType)[number];
export const ParameterExplicitType = [
    DiagnosticCode.ParameterMutable,
    DiagnosticCode.ParameterImmutable,
    DiagnosticCode.ParameterMutableRest,
    DiagnosticCode.ParameterImmutableRest,
] as const;
/** 子模式参数类型 */
export type ParameterSubPatternType = (typeof ParameterSubPatternType)[number];
export const ParameterSubPatternType = [
    DiagnosticCode.ParameterSubPatternImmutable,
    DiagnosticCode.ParameterSubPatternMutable,
] as const;
/** 模式参数类型 */
export type ParameterPatternType = (typeof ParameterPatternType)[number];
export const ParameterPatternType = [DiagnosticCode.ParameterPattern, DiagnosticCode.ParameterRestPattern] as const;
/** 隐式参数类型 */
export type ParameterItType = (typeof ParameterItType)[number];
export const ParameterItType = [DiagnosticCode.ParameterIt] as const;
/** 参数定义类型 */
export type ParameterDefinitionType = (typeof ParameterDefinitionType)[number];
export const ParameterDefinitionType = [
    ...ParameterExplicitType,
    ...ParameterSubPatternType,
    ...ParameterItType,
] as const;
/** 参数占位符类型 */
export type ParameterPlaceholderType = (typeof ParameterPlaceholderType)[number];
export const ParameterPlaceholderType = [
    ...ParameterExplicitType,
    ...ParameterPatternType,
    ...ParameterItType,
] as const;
/** 局部定义类型 */
export type LocalDefinitionType = (typeof LocalDefinitionType)[number];
export const LocalDefinitionType = [...LocalVariableType, ...LocalFunctionType, ...ParameterDefinitionType] as const;

/** 源代码定义信息 */
export interface LocalDefinition<T extends LocalDefinitionType = LocalDefinitionType>
    extends SourceDefinitionBase<
        | DiagnosticCode.ReadLocal
        | DiagnosticCode.WriteLocal
        | DiagnosticCode.ReadWriteLocal
        | DiagnosticCode.RedeclareLocal
    > {
    /** 符号定义 */
    readonly definition: SourceDiagnostic<T>;
    /** 定义的函数，仅对 LocalFunction 有效 */
    readonly fn?: {
        /** 函数作用域 */
        scope: SourceScope;
        /** 函数的参数 */
        args: ReadonlyArray<LocalDefinition<ParameterDefinitionType>>;
    };
}
/** 源代码定义信息 */
export interface GlobalDefinition extends SourceDefinitionBase<DiagnosticCode.GlobalVariable> {
    /** 符号名称 */
    readonly name: string;
}
/** 源代码定义信息 */
export type SourceDefinition = LocalDefinition | GlobalDefinition;

/** 作用域信息 */
export interface SourceScope {
    /** 作用域范围 */
    readonly range: IRange;
    /** 包含的局部变量 */
    readonly locals: readonly LocalDefinition[];
    /** 包含的参数占位符 */
    readonly params: ReadonlyArray<SourceDiagnostic<ParameterPlaceholderType>>;
    /** 包含的作用域 */
    readonly children: readonly SourceScope[];
    /** 父作用域 */
    readonly parent?: SourceScope;
}

/** 编译结果 */
export class CompileResult {
    constructor(
        /** URI */
        readonly uri: string,
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
    get errors(): readonly SourceDiagnostic[] {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._errors;
    }
    /** 源代码诊断信息 */
    get warnings(): readonly SourceDiagnostic[] {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._warnings;
    }
    /** 源代码诊断信息 */
    get infos(): readonly SourceDiagnostic[] {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._infos;
    }
    /** 源代码诊断信息 */
    get hints(): readonly SourceDiagnostic[] {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._hints;
    }
    /** 源代码诊断信息 */
    get tags(): readonly SourceDiagnostic[] {
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
                if (
                    (ref.code > DiagnosticCode.ReferenceStart && ref.code < DiagnosticCode.ReferenceEnd) ||
                    (ref.code > DiagnosticCode.TagRefStart && ref.code < DiagnosticCode.TagRefEnd)
                ) {
                    i++;
                    ref.diagnostic = diagnostic;
                    (diagnostic.references as SourceReference[]).push(ref);
                } else {
                    break;
                }
            }
        }
        this.diagnosticsReady = true;
    }

    private _groupedTags?: {
        locals: readonly LocalDefinition[];
        params: ReadonlyArray<SourceDiagnostic<ParameterPlaceholderType>>;
        globals: readonly GlobalDefinition[];
        ranges: readonly SourceDiagnostic[];
        omitNameFields: ReadonlyArray<SourceDiagnostic<DiagnosticCode.OmitNamedRecordField>>;
    };

    /** 获取源代码定义 */
    groupedTags(model: editor.ITextModel): NonNullable<typeof this._groupedTags> {
        if (this._groupedTags) {
            return this._groupedTags;
        }
        const getText = (range: IRange): string | undefined => {
            if (model.getVersionId() !== this.version) {
                return undefined;
            }
            return model.getValueInRange(range);
        };
        const locals: Array<Writable<Partial<LocalDefinition>>> = [];
        const params: Array<SourceDiagnostic<ParameterPlaceholderType>> = [];
        const globals: Array<Writable<Partial<GlobalDefinition>>> = [];
        const ranges: Array<Writable<SourceDiagnostic>> = [];
        const omitNameFields: Array<SourceDiagnostic<DiagnosticCode.OmitNamedRecordField>> = [];
        for (const tag of this.tags) {
            // 可能与其他条件重叠
            if (ParameterPlaceholderType.includes(tag.code as ParameterPlaceholderType)) {
                // 参数占位符
                params.push(tag as SourceDiagnostic<ParameterPlaceholderType>);
            }

            if (LocalDefinitionType.includes(tag.code as LocalDefinitionType)) {
                locals.push({
                    definition: tag as SourceDiagnostic<never>,
                    references: tag.references as Array<SourceReference<never>>,
                });
            } else if (tag.code === DiagnosticCode.GlobalVariable) {
                const name = getText(tag.range);
                let def = globals.find((def) => name === def.name);
                if (!def) {
                    def = {
                        name: name ?? '',
                        references: [],
                    };
                    globals.push(def);
                }
                (def.references as SourceDiagnostic[]).push(tag);
            } else if (
                tag.code === DiagnosticCode.Scope ||
                tag.code === DiagnosticCode.String ||
                tag.code === DiagnosticCode.Interpolation
            ) {
                ranges.push(tag);
            } else if (tag.code === DiagnosticCode.OmitNamedRecordField) {
                omitNameFields.push(tag as SourceDiagnostic<DiagnosticCode.OmitNamedRecordField>);
            }
        }

        this._groupedTags = {
            locals: (locals as LocalDefinition[]).sort((a, b) =>
                Range.compareRangesUsingStarts(a.definition.range, b.definition.range),
            ),
            params: params.sort((a, b) => Range.compareRangesUsingStarts(a.range, b.range)),
            globals: globals as GlobalDefinition[],
            ranges: (ranges as SourceDiagnostic[]).sort((a, b) => Range.compareRangesUsingStarts(a.range, b.range)),
            omitNameFields: omitNameFields,
        };
        return this._groupedTags;
    }

    /** 获取定义 */
    definition(
        model: editor.ITextModel,
        position: IPosition,
    ): { def: LocalDefinition; ref?: number } | { def: GlobalDefinition; ref: number } | undefined {
        const { globals } = this.groupedTags(model);
        for (const d of globals) {
            const refIndex = d.references.findIndex((u) => strictContainsPosition(u.range, position));
            if (refIndex >= 0) {
                return { def: d, ref: refIndex };
            }
        }
        this.scopes(model); // 确保作用域信息已加载
        const { locals } = this.groupedTags(model);
        for (const d of locals) {
            if (strictContainsPosition(d.definition.range, position)) {
                return { def: d, ref: undefined };
            }
            const refIndex = d.references.findIndex((u) => strictContainsPosition(u.range, position));
            if (refIndex >= 0) {
                return { def: d, ref: refIndex };
            }
        }
        return undefined;
    }

    private _scopes?: readonly SourceScope[];
    private _scopeMap?: Map<LocalDefinition, SourceScope>;
    /** 获取作用域信息 */
    scopes(model: editor.ITextModel): readonly SourceScope[] {
        if (this._scopes) {
            return this._scopes;
        }
        const { locals, params, ranges } = this.groupedTags(model);

        // 1. 提取所有 Scope 范围
        const scopes = ranges
            .filter((r) => r.code === DiagnosticCode.Scope)
            .map((r) => {
                return {
                    range: r.range,
                    locals: [],
                    params: [],
                    parent: undefined,
                    children: [],
                } as Writable<SourceScope>;
            });

        // 2. 按范围嵌套关系构建父子树
        for (let i = 0; i < scopes.length; i++) {
            const scopeA = scopes[i]!;
            let parent: Writable<SourceScope> | undefined;
            for (let j = 0; j < scopes.length; j++) {
                if (i === j) continue;
                const scopeB = scopes[j]!;
                // 判断 scopeA 是否被 scopeB 包含（严格包含）
                const aRange = scopeA.range;
                const bRange = scopeB.range;
                const isContained = Range.containsRange(bRange, aRange);
                if (isContained) {
                    // 选择最近的父作用域
                    if (!parent || Range.containsRange(parent.range, bRange)) {
                        parent = scopeB;
                    }
                }
            }
            if (parent) {
                scopeA.parent = parent;
                (parent.children as Writable<SourceScope[]>).push(scopeA);
            }
        }

        // 3. 按 BFS 顺序排序作用域
        const root = scopes.find((s) => !s.parent);
        // 由于脚本本身就是一个作用域，所以根作用域一定存在且唯一
        if (!root) {
            // Fail safe, return empty scopes
            this._scopes = [];
            return this._scopes;
        }
        const queue: SourceScope[] = [root];
        const sortedScopes: SourceScope[] = [];
        while (queue.length > 0) {
            const scope = queue.shift()!;
            sortedScopes.push(scope);
            (scope.children as Writable<SourceScope[]>).sort((a, b) =>
                Range.compareRangesUsingStarts(a.range, b.range),
            );
            for (const child of scope.children) {
                queue.push(child);
            }
        }

        // 4. 填充每个作用域的局部变量
        for (const local of locals) {
            const { range } = local.definition;
            const scope = sortedScopes.findLast((s) => Range.containsRange(s.range, range));
            if (scope) {
                (scope.locals as Writable<LocalDefinition[]>).push(local);
            }
        }
        for (const param of params) {
            const { range } = param;
            const scope = sortedScopes.findLast((s) => Range.containsRange(s.range, range));
            if (scope) {
                (scope.params as Array<SourceDiagnostic<ParameterPlaceholderType>>).push(param);
            }
        }

        // 5. 构建作用域映射
        const scopeMap = new Map<LocalDefinition, SourceScope>();
        for (const scope of sortedScopes) {
            (scope.locals as Writable<LocalDefinition[]>).sort((a, b) =>
                Range.compareRangesUsingStarts(a.definition.range, b.definition.range),
            );
            (scope.params as Array<SourceDiagnostic<ParameterPlaceholderType>>).sort((a, b) =>
                Range.compareRangesUsingStarts(a.range, b.range),
            );
            for (const local of scope.locals) {
                scopeMap.set(local, scope);
                if (local.definition.code === DiagnosticCode.LocalFunction) {
                    const funcScope = scope.children.find(
                        (s) => Range.compareRangesUsingStarts(s.range, local.definition.range) > 0,
                    );
                    if (funcScope) {
                        const args = funcScope.locals.filter((l): l is LocalDefinition<ParameterDefinitionType> =>
                            ParameterDefinitionType.includes(l.definition.code as ParameterDefinitionType),
                        );
                        (local as Writable<LocalDefinition>).fn = {
                            scope: funcScope,
                            args,
                        };
                    }
                }
            }
        }

        this._scopeMap = scopeMap;
        this._scopes = sortedScopes;
        return this._scopes;
    }

    /** 获取定义所在作用域 */
    scopeOf(model: editor.ITextModel, def: LocalDefinition): SourceScope | undefined {
        if (!this._scopeMap) {
            this.scopes(model);
        }
        return this._scopeMap!.get(def);
    }

    /** 获取位置所在作用域 */
    scopeAt(model: editor.ITextModel, position: IPosition): SourceScope {
        const scopes = this.scopes(model);
        let scope = scopes.findLast((s) => Range.containsPosition(s.range, position)) ?? scopes[0]!; // 失败时从根作用域开始查找
        while (scope.children.length > 0) {
            const inner = scope.children.find((s) => Range.containsPosition(s.range, position));
            if (!inner) break;
            scope = inner;
        }
        return scope;
    }
}
