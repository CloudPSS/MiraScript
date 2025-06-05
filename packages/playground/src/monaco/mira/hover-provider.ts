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
import { VmSharedGlobal } from '../../vm/types/global.js';
import { getVmFunctionInfo } from '../../vm';
import { $ToString } from '../../vm/operations';
import { signature, codeblock } from './utils';

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
        for (const diagnostic of compiled.diagnostics) {
            let content: IMarkdownString | undefined;
            // eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
            switch (diagnostic.code) {
                case DiagnosticCode.ParameterImmutable:
                    content = {
                        value: codeblock(`\0(parameter) ${model.getValueInRange(diagnostic)}`),
                    };
                    break;
                case DiagnosticCode.ParameterMutable:
                    content = {
                        value: codeblock(`\0(parameter) mut ${model.getValueInRange(diagnostic)}`),
                    };
                    break;
                case DiagnosticCode.ParameterImmutableRest:
                    content = {
                        value: codeblock(`\0(parameter) ..${model.getValueInRange(diagnostic)}`),
                    };
                    break;
                case DiagnosticCode.ParameterMutableRest:
                    content = {
                        value: codeblock(`\0(parameter) ..mut ${model.getValueInRange(diagnostic)}`),
                    };
                    break;
                case DiagnosticCode.LocalImmutable:
                    content = {
                        value: codeblock(`let ${model.getValueInRange(diagnostic)}`),
                    };
                    break;
                case DiagnosticCode.LocalMutable:
                    content = {
                        value: codeblock(`let mut ${model.getValueInRange(diagnostic)}`),
                    };
                    break;
                case DiagnosticCode.GlobalDynamicAccess: {
                    const expr = model.getValueInRange(diagnostic);
                    content = {
                        value: codeblock(`\0(global) [${expr}]`),
                    };
                    break;
                }
                case DiagnosticCode.GlobalVariable: {
                    const id = model.getValueInRange(diagnostic);
                    const value = VmSharedGlobal[id];
                    let description = id;
                    let doc = '';
                    const info = getVmFunctionInfo(value);
                    if (info) {
                        description = signature(id, info);
                        doc = info.summary ?? '';
                    } else if (value !== undefined) {
                        description = `${id} = ${$ToString(value)}`;
                    }
                    content = {
                        value: codeblock(`\0(global) ${description}`) + doc,
                    };
                    break;
                }
                default:
                    continue;
            }
            if (!Range.containsPosition(diagnostic, position)) {
                continue;
            }
            if (hover.range) {
                hover.range = Range.intersectRanges(hover.range, diagnostic) ?? undefined;
            } else {
                hover.range = Range.lift(diagnostic);
            }
            hover.contents.push(content);
        }
        return hover;
    }
}
languages.registerHoverProvider('mirascript', new HoverProvider());
