import type { CancellationToken, editor, IMarkdownString, IRange, languages, Position } from '../../monaco-api.js';
import { Provider } from './base.js';
import { DiagnosticCode } from '@mirascript/constants';
import { codeblock, getDeep, valueDoc, paramsList } from '../utils.js';
import type { FieldsAccessAt, VariableAccessAt } from '../compile-result.js';
import { KEYWORDS as HELP_KEYWORDS, OPERATORS as HELP_OPERATORS } from '@mirascript/help';

const OPERATOR_TOKENS_DESC = Object.keys(HELP_OPERATORS as Record<string, string>).sort((a, b) => b.length - a.length);

/** 在指定位置查找操作符 */
function operatorAt(lineContent: string, column: number): { token: string; range: IRange } | undefined {
    const index = Math.max(0, column - 1);
    for (const token of OPERATOR_TOKENS_DESC) {
        for (let offset = 0; offset < token.length; offset++) {
            const start = index - offset;
            if (start < 0) continue;
            const end = start + token.length;
            if (end > lineContent.length) continue;
            if (lineContent.slice(start, end) !== token) continue;
            if (index < start || index >= end) continue;
            return {
                token,
                range: {
                    startLineNumber: 0,
                    startColumn: start + 1,
                    endLineNumber: 0,
                    endColumn: end + 1,
                },
            };
        }
    }
    return undefined;
}

/** @inheritdoc */
export class HoverProvider extends Provider implements languages.HoverProvider {
    /** 变量提示 */
    private async provideVariableHover(
        model: editor.ITextModel,
        { def, ref }: VariableAccessAt,
    ): Promise<languages.Hover | undefined> {
        const contents: IMarkdownString[] = [];
        let range: IRange | undefined;
        if ('name' in def) {
            const globals = await this.getContext(model);
            const value = globals.getOrUndefined(def.name);
            const { script, doc } = valueDoc(def.name, value, 'hint', globals);
            contents.push({ value: codeblock(`\0(global) ${script}`) });
            for (const d of doc) {
                contents.push({ value: d });
            }
            range = def.references[ref!]?.range;
        } else {
            let content: IMarkdownString | undefined;
            const tag = def.definition;
            switch (tag.code) {
                case DiagnosticCode.ParameterSubPatternImmutable:
                    content = {
                        value: codeblock(`\0(parameter pattern) ${model.getValueInRange(tag.range)}`),
                    };
                    break;
                case DiagnosticCode.ParameterSubPatternMutable:
                    content = {
                        value: codeblock(`\0(parameter pattern) mut ${model.getValueInRange(tag.range)}`),
                    };
                    break;
                case DiagnosticCode.ParameterImmutable:
                    content = {
                        value: codeblock(`\0(parameter) ${model.getValueInRange(tag.range)}`),
                    };
                    break;
                case DiagnosticCode.ParameterMutable:
                    content = {
                        value: codeblock(`\0(parameter) mut ${model.getValueInRange(tag.range)}`),
                    };
                    break;
                case DiagnosticCode.ParameterIt:
                    content = {
                        value: codeblock(`\0(parameter) it`),
                    };
                    break;
                case DiagnosticCode.ParameterImmutableRest:
                    content = {
                        value: codeblock(`\0(parameter) ..${model.getValueInRange(tag.range)}`),
                    };
                    break;
                case DiagnosticCode.ParameterMutableRest:
                    content = {
                        value: codeblock(`\0(parameter) ..mut ${model.getValueInRange(tag.range)}`),
                    };
                    break;
                case DiagnosticCode.LocalFunction: {
                    const params = paramsList(model, def.fn);
                    content = {
                        value: codeblock(`\0fn ${model.getValueInRange(tag.range)}${params}`),
                    };
                    break;
                }
                case DiagnosticCode.LocalImmutable:
                    content = {
                        value: codeblock(`\0let ${model.getValueInRange(tag.range)}`),
                    };
                    break;
                case DiagnosticCode.LocalConst:
                    content = {
                        value: codeblock(`\0const ${model.getValueInRange(tag.range)}`),
                    };
                    break;
                case DiagnosticCode.LocalMutable:
                    content = {
                        value: codeblock(`\0let mut ${model.getValueInRange(tag.range)}`),
                    };
                    break;
            }
            if (ref == null) {
                range = tag.range;
            } else {
                range = def.references[ref]?.range;
            }
            if (content) {
                contents.push(content);
            }
        }
        if (!contents.length) return undefined;
        return {
            contents,
            range: range,
        };
    }

    /** 字段提示 */
    private async provideFieldHover(
        model: editor.ITextModel,
        range: IRange,
        { def: { def, ref }, fields }: FieldsAccessAt,
    ): Promise<languages.Hover | undefined> {
        if ('definition' in def) {
            // TODO: provide local item fields
            return undefined;
        }
        const vmGlobal = await this.getContext(model);
        const lastField = fields.at(-1)!;
        const [obj, value] = getDeep(vmGlobal, def.name, fields);
        if (value == null) return undefined;
        const { script, doc } = valueDoc(lastField, value, 'field', obj);
        return {
            contents: [{ value: codeblock(`\0(field) ${script}`) }, ...doc.map((d) => ({ value: d }))],
            range,
        };
    }

    /** 语法元素提示 */
    private provideSyntaxHover(model: editor.ITextModel, position: Position): languages.Hover | undefined {
        const word = model.getWordAtPosition(position);
        if (word) {
            const doc = (HELP_KEYWORDS as Record<string, string | undefined>)[word.word];
            if (doc) {
                return {
                    contents: [{ value: doc }],
                    range: {
                        startLineNumber: position.lineNumber,
                        endLineNumber: position.lineNumber,
                        startColumn: word.startColumn,
                        endColumn: word.endColumn,
                    },
                };
            }
        }

        const lineContent = model.getLineContent(position.lineNumber);
        const hit = operatorAt(lineContent, position.column);
        if (hit) {
            const doc = (HELP_OPERATORS as Record<string, string | undefined>)[hit.token];
            if (doc) {
                return {
                    contents: [{ value: doc }],
                    range: {
                        startLineNumber: position.lineNumber,
                        endLineNumber: position.lineNumber,
                        startColumn: hit.range.startColumn,
                        endColumn: hit.range.endColumn,
                    },
                };
            }
        }

        return undefined;
    }

    /** @inheritdoc */
    async provideHover(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
        context?: languages.HoverContext<languages.Hover>,
    ): Promise<languages.Hover | undefined> {
        const value = await this.getValueAt(model, position);
        if (!value) {
            return this.provideSyntaxHover(model, position);
        } else if ('fields' in value) {
            return this.provideFieldHover(model, value.range, value.fields);
        } else {
            return this.provideVariableHover(model, value.variable);
        }
    }
}
