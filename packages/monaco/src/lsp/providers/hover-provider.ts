import type { CancellationToken, editor, IMarkdownString, IRange, languages, Position } from '../../monaco-api.js';
import { Provider } from './base.js';
import { DiagnosticCode } from '@mirascript/bindings/wasm';
import { codeblock, getDeep, valueDoc, paramsList } from '../utils.js';
import type { FieldsAccessAt, VariableAccessAt } from '../compile-result.js';

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
            const value = globals.has(def.name) ? globals.get(def.name) : undefined;
            const { script, doc } = valueDoc(def.name, value, 'hint');
            contents.push({ value: codeblock(`\0(global) ${script}`) });
            for (const d of doc) {
                contents.push({ value: d });
            }
            const describe = globals.describe?.(def.name);
            if (describe) {
                contents.push({ value: describe });
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
        const value = getDeep(vmGlobal.get(def.name), fields);
        if (value == null) return undefined;
        const lastField = fields.pop()!;
        const { script, doc } = valueDoc(lastField, value, 'field');
        return {
            contents: [{ value: codeblock(`\0(field) ${script}`) }, ...doc.map((d) => ({ value: d }))],
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
        const value = await this.getValueAt(model, position);
        if (!value) return undefined;
        if ('fields' in value) {
            return this.provideFieldHover(model, value.range, value.fields);
        } else {
            return this.provideVariableHover(model, value.variable);
        }
    }
}
