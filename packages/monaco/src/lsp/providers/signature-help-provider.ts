import { DiagnosticCode } from '@mirascript/wasm';
import { type editor, type languages, type CancellationToken, Position, Range } from '../../monaco-api.js';
import { Provider } from './base.js';
import { getDeep, localParamList, paramSignature, strictContainsPosition, valueDoc } from '../utils.js';
import { getVmFunctionInfo } from '@mirascript/mirascript';

/** @inheritdoc */
export class SignatureHelpProvider extends Provider implements languages.SignatureHelpProvider {
    /** @inheritdoc */
    readonly signatureHelpTriggerCharacters = ['(', ','];
    /** @inheritdoc */
    readonly signatureHelpRetriggerCharacters = [')'];
    /** @inheritdoc */
    async provideSignatureHelp(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
        context: languages.SignatureHelpContext,
    ): Promise<languages.SignatureHelpResult | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;

        const invokes = compiled.groupedTags(model).ranges.filter((r) => {
            if (r.code !== DiagnosticCode.FunctionCall && r.code !== DiagnosticCode.ExtensionCall) {
                return false;
            }
            if (!strictContainsPosition(r.range, position)) {
                return false;
            }
            const argStart = r.references.find((ref) => ref.code === DiagnosticCode.ArgumentStart);
            const argEnd = r.references.find((ref) => ref.code === DiagnosticCode.ArgumentEnd);
            if (!argStart || !argEnd) {
                return false;
            }
            return Range.containsPosition(
                Range.fromPositions(Range.getEndPosition(argStart.range), Range.getStartPosition(argEnd.range)),
                position,
            );
        });
        if (!invokes.length) return undefined;
        // 获取最内层的调用
        invokes.sort((a, b) => (Range.strictContainsRange(a.range, b.range) ? 1 : -1));
        const invoke = invokes[0]!;

        const callableRef = invoke.references.find((ref) => ref.code === DiagnosticCode.Callable);
        if (!callableRef) return undefined;
        const callableInfo = compiled.accessAt(model, Range.getEndPosition(callableRef.range));
        if (!callableInfo) return undefined;

        const signature: languages.SignatureInformation = {
            label: '',
            parameters: [],
        };
        if ('name' in callableInfo.def.def) {
            const globals = await this.getContext(model);
            const callable = getDeep(globals.get(callableInfo.def.def.name), callableInfo.fields);
            const info = getVmFunctionInfo(callable);
            if (!info) return undefined;
            const doc = valueDoc(callableInfo.def.def.name, callable, false);
            signature.label = doc.script;
            signature.documentation = {
                value: doc.doc,
            };
            for (const p of Object.keys(info.paramsType ?? {})) {
                signature.parameters.push({
                    label: paramSignature(p, info),
                });
            }
        } else if (callableInfo.def.def.fn) {
            const { fn, definition } = callableInfo.def.def;
            const params = localParamList(model, fn);
            signature.label = `fn ${model.getValueInRange(definition.range)}(${params.join(', ')})`;
            for (const p of params) {
                signature.parameters.push({ label: p });
            }
        }

        if (!signature.label) return undefined;
        let pos = invoke.code === DiagnosticCode.FunctionCall ? 0 : 1;
        for (const ref of invoke.references) {
            if (ref.code === DiagnosticCode.ArgumentSpread) pos = Number.NaN;
            if (ref.code !== DiagnosticCode.ArgumentComma || Range.isEmpty(ref.range)) continue;
            if (Position.isBeforeOrEqual(Range.getEndPosition(ref.range), position)) {
                pos++;
            }
        }

        return {
            dispose: () => void 0,
            value: {
                signatures: [signature],
                activeSignature: 0,
                activeParameter: pos,
            },
        };
    }
}
