import {
    type CancellationToken,
    type editor,
    type IMarkdownString,
    languages,
    type Position,
} from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { DiagnosticCode } from 'mira-wasm';
import { codeblock, getGlobal, paramsList } from './utils';
import type { LocalDefinition } from './compile-result';

/** @inheritdoc */
class HoverProvider extends Provider implements languages.HoverProvider {
    /** @inheritdoc */
    async provideHover(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
        context?: languages.HoverContext<languages.Hover>,
    ): Promise<languages.Hover | undefined> {
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) {
            return undefined;
        }
        const def = compiled.definition(model, position);
        if (!def) return undefined;
        let content: IMarkdownString | undefined;
        const tag = def.ref != null ? def.def.references[def.ref]! : def.def.definition;
        switch (tag.code) {
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
            case DiagnosticCode.UnusedParameterIt:
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
                const params = paramsList(model, (def.def as LocalDefinition).fn);
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
            case DiagnosticCode.GlobalVariable: {
                const id = model.getValueInRange(tag.range);
                const { script, doc } = getGlobal(id);
                content = {
                    value: codeblock(`\0(global) ${script}`) + doc,
                };
                break;
            }
        }
        if (!content) return undefined;
        return {
            contents: [content],
            range: tag.range,
        };
    }
}
languages.registerHoverProvider('mirascript', new HoverProvider());
