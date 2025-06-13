import type { CancellationToken, editor, IMarkdownString, IRange, languages, Position } from '../../monaco-api.js';
import { Provider } from './base.js';
import { DiagnosticCode } from '@mirascript/wasm';
import { codeblock, getGlobal, paramsList } from '../utils';
import type { LocalDefinition } from '../compile-result';

/** @inheritdoc */
export class HoverProvider extends Provider implements languages.HoverProvider {
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
        const d = compiled.definition(model, position);
        if (!d) return undefined;
        const { def, ref } = d;
        let content: IMarkdownString | undefined;
        let range: IRange | undefined;
        if ('name' in def) {
            const { script, doc } = getGlobal(def.name);
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
                    const params = paramsList(model, (d.def as LocalDefinition).fn);
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
}
