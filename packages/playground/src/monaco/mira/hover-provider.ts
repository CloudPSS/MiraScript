import {
    type CancellationToken,
    type editor,
    type IMarkdownString,
    languages,
    type Position,
    Range,
} from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { DiagnosticCode } from 'mira-wasm';
import { codeblock, getGlobalScript } from './utils';

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
        const hover: languages.Hover = {
            contents: [],
        };
        for (const tag of compiled.tags) {
            if (!Range.containsPosition(tag.range, position)) {
                continue;
            }
            let content: IMarkdownString | undefined;
            // eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
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
                case DiagnosticCode.LocalFunction:
                    content = {
                        value: codeblock(`\0fn ${model.getValueInRange(tag.range)}`),
                    };
                    break;
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
                case DiagnosticCode.GlobalDynamicAccess: {
                    const expr = model.getValueInRange(tag.range);
                    content = {
                        value: codeblock(`\0(global) [${expr}]`),
                    };
                    break;
                }
                case DiagnosticCode.GlobalVariable: {
                    const id = model.getValueInRange(tag.range);
                    const { script, doc } = getGlobalScript(id);
                    content = {
                        value: codeblock(`\0(global) ${script}`) + doc,
                    };
                    break;
                }
                default:
                    continue;
            }
            if (hover.range) {
                hover.range = Range.intersectRanges(hover.range, tag.range) ?? undefined;
            } else {
                hover.range = Range.lift(tag.range);
            }
            hover.contents.push(content);
        }
        return hover;
    }
}
languages.registerHoverProvider('mirascript', new HoverProvider());
