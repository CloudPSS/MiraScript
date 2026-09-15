import { DiagnosticCode, type SourceReference } from '@mirascript/mirascript/subtle';
import { type IRange, Range } from '../../monaco-api.js';
import type { SourceDiagnostic } from './index.js';
import {
    LocalDefinitionType,
    ParameterPlaceholderType,
    type GlobalDefinition,
    type LocalDefinition,
} from './interfaces.js';

/** 源代码定义 */
export interface GroupedTags {
    /** 本地变量标签 */
    locals: readonly LocalDefinition[];
    /** 参数标签 */
    params: ReadonlyArray<SourceDiagnostic<ParameterPlaceholderType>>;
    /** 全局变量标签 */
    globals: readonly GlobalDefinition[];
    /** 范围标签 */
    ranges: readonly SourceDiagnostic[];
    /** 省略命名记录字段标签 */
    omitNameFields: ReadonlyArray<SourceDiagnostic<DiagnosticCode.OmitNamedRecordField>>;
}

/** 获取源代码定义 */
export function groupTags(
    tags: readonly SourceDiagnostic[],
    getText: (range: IRange) => string | undefined,
): GroupedTags {
    const locals: Array<Writable<Partial<LocalDefinition>>> = [];
    const params: Array<SourceDiagnostic<ParameterPlaceholderType>> = [];
    const globals: Array<Writable<Partial<GlobalDefinition>>> = [];
    const ranges: Array<Writable<SourceDiagnostic>> = [];
    const omitNameFields: Array<SourceDiagnostic<DiagnosticCode.OmitNamedRecordField>> = [];
    for (const tag of tags) {
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
            tag.code === DiagnosticCode.Interpolation ||
            tag.code === DiagnosticCode.FunctionCall ||
            tag.code === DiagnosticCode.ExtensionCall
        ) {
            ranges.push(tag);
        } else if (tag.code === DiagnosticCode.OmitNamedRecordField) {
            omitNameFields.push(tag as SourceDiagnostic<DiagnosticCode.OmitNamedRecordField>);
        }
    }

    (locals as LocalDefinition[]).sort((a, b) =>
        Range.compareRangesUsingStarts(a.definition.range, b.definition.range),
    );
    params.sort((a, b) => Range.compareRangesUsingStarts(a.range, b.range));
    (globals as GlobalDefinition[]).sort((a, b) => a.name.localeCompare(b.name));
    ranges.sort((a, b) => Range.compareRangesUsingStarts(a.range, b.range));

    return {
        locals: locals as LocalDefinition[],
        params: params,
        globals: globals as GlobalDefinition[],
        ranges: ranges,
        omitNameFields: omitNameFields,
    };
}
