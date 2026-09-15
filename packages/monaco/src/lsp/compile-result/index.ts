import type { Writable } from 'type-fest';
import { type editor, Range, type IRange, type IPosition, Position } from '../../monaco-api.js';
import { strictContainsPosition } from '../monaco-utils.js';
import { REG_IDENTIFIER_FULL, REG_ORDINAL_FULL } from '../../constants.js';
import type { CacheKey, MonacoResult } from '../worker-core.js';
import {
    parseDiagnostics,
    DiagnosticCode,
    type SourceDiagnostic,
    type SourceReference,
} from '@mirascript/mirascript/subtle';
import {
    LocalVariableType,
    ParameterExplicitType,
    ParameterSubPatternType,
    ParameterPatternType,
    ParameterItType,
    ParameterDefinitionType,
    ParameterPlaceholderType,
    type LocalDefinition,
    type GlobalDefinition,
    type SourceDefinition,
    type SourceScope,
} from './interfaces.js';
import { groupTags, type GroupedTags } from './grouped-tags.js';
import { makeScopes, type ScopeInfo } from './scopes.js';

export {
    LocalVariableType,
    ParameterExplicitType,
    ParameterSubPatternType,
    ParameterPatternType,
    ParameterItType,
    ParameterDefinitionType,
    ParameterPlaceholderType,
};
export type { SourceDiagnostic, SourceReference, LocalDefinition, GlobalDefinition, SourceDefinition, SourceScope };

/** 变量访问 */
export type VariableAccessAt = {
    /** 访问发生的位置 */
    range: IRange;
} & (
    | {
          /** 访问的变量定义 */
          def: LocalDefinition;
          /** 访问的是哪一个引用，`undefined` 表示访问了定义 */
          ref?: number;
      }
    | {
          /** 访问的变量定义 */
          def: GlobalDefinition;
          /** 访问的是哪一个引用 */
          ref: number;
      }
);

/** 字段访问 */
export type FieldsAccessAt = {
    /** 访问的变量 */
    def: VariableAccessAt;
    /** 访问的字段 */
    fields: string[];
};

/** 编译结果 */
export class CompileResult {
    constructor(
        /** CacheKey */
        readonly cacheKey: CacheKey,
        /** 代码版本 */
        readonly version: number,
        readonly source: string,
        readonly result: MonacoResult,
    ) {
        this.diagnostics = result.diagnostics;
        this.chunk = result.chunk;
    }
    /** 源代码诊断信息 */
    private readonly diagnostics: Uint32Array;
    /** 代码信息 */
    readonly chunk?: Uint8Array;

    private diagnosticsReady = false;
    private _errors: Array<Writable<SourceDiagnostic>> = [];
    private _warnings: Array<Writable<SourceDiagnostic>> = [];
    private _infos: Array<Writable<SourceDiagnostic>> = [];
    private _hints: Array<Writable<SourceDiagnostic>> = [];
    private _references: Array<Writable<SourceReference>> = [];
    private _tags: Array<Writable<SourceDiagnostic>> = [];
    private _tagsReferences: Array<Writable<SourceReference>> = [];
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
    get references(): readonly SourceReference[] {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._references;
    }
    /** 源代码诊断信息 */
    get tags(): readonly SourceDiagnostic[] {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._tags;
    }
    /** 源代码诊断信息 */
    get tagsReferences(): readonly SourceReference[] {
        if (!this.diagnosticsReady) {
            this.readDiagnostics();
        }
        return this._tagsReferences;
    }
    /** 分析诊断信息 */
    private readDiagnostics(): void {
        const parsed = parseDiagnostics(this.source, this.diagnostics, (c) => c !== DiagnosticCode.SourceMap);
        this._errors = parsed.errors;
        this._warnings = parsed.warnings;
        this._infos = parsed.infos;
        this._hints = parsed.hints;
        this._tags = parsed.tags;
        this._references = parsed.references;
        this._tagsReferences = parsed.tagsReferences;
        this.diagnosticsReady = true;
    }

    private _groupedTags?: GroupedTags;

    /** 获取源代码定义 */
    groupedTags(model: editor.ITextModel): GroupedTags {
        if (this._groupedTags) {
            return this._groupedTags;
        }

        this._groupedTags = groupTags(this._tags, (range) => {
            if (model.getVersionId() !== this.version) {
                return undefined;
            }
            return model.getValueInRange(range);
        });
        return this._groupedTags;
    }

    /** 获取指定位置的变量访问信息 */
    variableAccessAt(model: editor.ITextModel, position: IPosition): VariableAccessAt | undefined {
        const getRefAccess = <T extends LocalDefinition | GlobalDefinition>(
            def: T,
        ): { def: T; ref: number; range: IRange } | undefined => {
            const refIndex = def.references.findIndex((u) => strictContainsPosition(u.range, position));
            if (refIndex < 0) return undefined;
            return { def, ref: refIndex, range: def.references[refIndex]!.range };
        };
        const { globals } = this.groupedTags(model);
        for (const d of globals) {
            const access = getRefAccess(d);
            if (access) return access;
        }
        this.scopes(model); // 确保作用域信息已加载
        const { locals } = this.groupedTags(model);
        for (const d of locals) {
            if (strictContainsPosition(d.definition.range, position)) {
                return { def: d, ref: undefined, range: d.definition.range };
            }
            const access = getRefAccess(d);
            if (access) return access;
        }
        return undefined;
    }

    private _scopeInfo?: ScopeInfo;
    /** 获取作用域信息 */
    scopes(model: editor.ITextModel): readonly SourceScope[] {
        if (this._scopeInfo) {
            return this._scopeInfo.scopes;
        }
        const groupedTags = this.groupedTags(model);
        this._scopeInfo = makeScopes(groupedTags, model.getFullModelRange());
        return this._scopeInfo.scopes;
    }

    /** 获取定义所在作用域 */
    scopeOf(model: editor.ITextModel, def: LocalDefinition): SourceScope | undefined {
        if (!this._scopeInfo) {
            this.scopes(model);
        }
        return this._scopeInfo!.map.get(def);
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

    /** 获取指定位置的字段访问信息 */
    fieldAccessAt(model: editor.ITextModel, position: IPosition): FieldsAccessAt | undefined {
        this.scopes(model); // 确保作用域信息已加载
        // 查找 position 前最近的一个变量
        let prevDef: VariableAccessAt | undefined;
        const readRefAccess = <T extends LocalDefinition | GlobalDefinition>(def: T): void => {
            for (const [refIndex, ref] of def.references.entries()) {
                if (!Position.isBefore(Range.getEndPosition(ref.range), position)) continue;
                if (prevDef && Position.isBefore(Range.getEndPosition(ref.range), Range.getEndPosition(prevDef.range)))
                    continue;
                prevDef = { def: def as LocalDefinition, ref: refIndex, range: ref.range };
            }
        };
        const { globals, locals } = this.groupedTags(model);
        for (const d of globals) {
            readRefAccess(d);
        }
        for (const d of locals) {
            readRefAccess(d);
        }
        if (!prevDef) return undefined;
        // 获取从变量开始到查找位置的源码
        const chain = model.getValueInRange(Range.fromPositions(Range.getStartPosition(prevDef.range), position));
        // 用 `!.` 和 `.` 切分
        const chainParts = chain.split(/\s*(?:!\.|\.)\s*/);
        if (
            // 至少包含变量名和当前位置的字段名
            chainParts.length < 2 ||
            !chainParts.every(
                (part, index) =>
                    // 如果是最后一个部分，则可以为空（表示当前位置的字段名），否则必须是合法的标识符
                    (index === chainParts.length - 1 ? !part : false) ||
                    REG_IDENTIFIER_FULL.test(part) ||
                    REG_ORDINAL_FULL.test(part),
            )
        ) {
            return undefined;
        }
        return { def: prevDef, fields: chainParts.slice(1) };
    }
    /** 获取指定位置的字段访问信息 */
    accessAt(model: editor.ITextModel, position: IPosition): FieldsAccessAt | undefined {
        const v = this.variableAccessAt(model, position);
        if (v) return { def: v, fields: [] };
        return this.fieldAccessAt(model, position);
    }
}
