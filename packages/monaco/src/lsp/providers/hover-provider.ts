import type { CancellationToken, editor, IMarkdownString, IRange, languages, Position } from '../../monaco-api.js';
import { Provider } from './base.js';
import { DiagnosticCode } from '@mirascript/wasm';
import { codeblock, getDeep, valueDoc, paramsList } from '../utils.js';
import type { AccessAt, DefinitionAt } from '../compile-result.js';

/** @inheritdoc */
export class HoverProvider extends Provider implements languages.HoverProvider {
    /** 变量提示 */
    private async provideVariableHover(
        model: editor.ITextModel,
        { def, ref }: DefinitionAt,
    ): Promise<languages.Hover | undefined> {
        let content: IMarkdownString | undefined;
        let range: IRange | undefined;
        if ('name' in def) {
            const globals = await this.getContext(model);
            const { script, doc } = valueDoc(def.name, globals[def.name], false);
            content = {
                value: codeblock(`\0(global) ${script}`) + doc,
            };
            range = def.references[ref!]?.range;
        } else {
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
        }
        if (!content) return undefined;
        return {
            contents: [content],
            range: range,
        };
    }

    /** 字段提示 */
    private async provideFieldHover(
        model: editor.ITextModel,
        range: IRange,
        { def: { def, ref }, fields }: AccessAt,
    ): Promise<languages.Hover | undefined> {
        if ('definition' in def) {
            // TODO: provide local item fields
            return undefined;
        }
        const vmGlobal = await this.getContext(model);
        const value = getDeep(vmGlobal[def.name], fields);
        if (value == null) return undefined;
        const lastField = fields.pop()!;
        const { script, doc } = valueDoc(lastField, value, true);
        return {
            contents: [
                {
                    value: codeblock(`\0(field) ${script}`) + doc,
                },
            ],
            range,
        };
    }
    /** @inheritdoc */
    async provideHover(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
        context?: languages.HoverContext<languages.Hover>,
    ): Promise<languages.Hover | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) {
            return undefined;
        }
        const d = compiled.definitionAt(model, position);
        if (d) {
            return this.provideVariableHover(model, d);
        }
        const word = model.getWordAtPosition(position);
        if (word) {
            const a = compiled.accessAt(model, {
                lineNumber: position.lineNumber,
                column: word.endColumn,
            });
            if (a) {
                return this.provideFieldHover(
                    model,
                    {
                        startColumn: word.startColumn,
                        endColumn: word.endColumn,
                        startLineNumber: position.lineNumber,
                        endLineNumber: position.lineNumber,
                    },
                    a,
                );
            }
        }
        return undefined;
    }
}
