import { DiagnosticCode } from '@mirascript/bindings/wasm';
import { type editor, type languages, type CancellationToken, Position, Range } from '../../monaco-api.js';
import { Provider } from './base.js';
import { fnSignature, getDeep, localParamSignature, strictContainsPosition } from '../utils.js';
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

        let sig;
        if ('name' in callableInfo.def.def) {
            const { name } = callableInfo.def.def;
            const globals = await this.getContext(model);
            const callable = getDeep(globals, name, callableInfo.fields);
            const info = getVmFunctionInfo(callable);
            if (!info) return undefined;
            sig = { ...fnSignature(name, info), name, summary: info.summary };
        } else if (callableInfo.def.def.fn) {
            const { fn, definition } = callableInfo.def.def;
            const params = localParamSignature(model, fn);
            const name = model.getValueInRange(definition.range);
            sig = { params, returns: '', name, summary: '' };
        }

        if (!sig) return undefined;
        const signature: languages.SignatureInformation = {
            label: '',
            parameters: [],
        };
        if (invoke.code === DiagnosticCode.ExtensionCall) {
            const thisArg = sig.params[0];
            if (thisArg && !thisArg[0].startsWith('..')) {
                sig.params.shift();
                const s = thisArg[1].includes(' ') ? `(${thisArg[1]})` : thisArg[1];
                signature.label = `fn ${s}::${sig.name}(`;
            } else {
                signature.label = `fn ()::${sig.name}(`;
            }
        } else {
            signature.label = `fn ${sig.name}(`;
        }
        signature.documentation = { value: sig.summary ?? '' };
        for (let i = 0; i < sig.params.length; i++) {
            const [_, p, doc] = sig.params[i]!;
            const start = signature.label.length;
            signature.parameters.push({
                label: [start, start + p.length],
                documentation: { value: doc },
            });
            if (i === sig.params.length - 1) {
                signature.label += p;
            } else {
                signature.label += p + ', ';
            }
        }
        signature.label += ')' + sig.returns;

        let pos = 0;
        for (const ref of invoke.references) {
            if (ref.code === DiagnosticCode.ArgumentSpread) pos = Number.NaN;
            if (ref.code !== DiagnosticCode.ArgumentComma || Range.isEmpty(ref.range)) continue;
            if (
                Position.isBeforeOrEqual(Range.getEndPosition(ref.range), position) &&
                !sig.params[pos]?.[0].startsWith('..')
            ) {
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
