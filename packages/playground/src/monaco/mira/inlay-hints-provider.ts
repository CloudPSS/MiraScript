import { type CancellationToken, type editor, type IEvent, languages, Range } from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { DiagnosticCode } from 'mira-wasm';

/** @inheritdoc */
class InlayHintsProvider extends Provider implements languages.InlayHintsProvider {
    /** @inheritdoc */
    get onDidChangeInlayHints(): IEvent<void> | undefined {
        return this.onDidChange;
    }
    /** @inheritdoc */
    async provideInlayHints(
        model: editor.ITextModel,
        range: Range,
        token: CancellationToken,
    ): Promise<languages.InlayHintList | null | undefined> {
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) {
            return undefined;
        }
        const hints: languages.InlayHint[] = [];
        for (const diagnostic of compiled.diagnostics) {
            let lineNumber = 0;
            let column = 0;
            let label = '';
            const kind = languages.InlayHintKind.Parameter;
            let paddingLeft = true;
            let paddingRight = false;
            const edits: languages.TextEdit[] = [];
            // eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
            switch (diagnostic.code) {
                case DiagnosticCode.OmittedFunctionArgument: {
                    lineNumber = diagnostic.endLineNumber;
                    column = diagnostic.endColumn;
                    label = '(it)';
                    paddingLeft = /\bfn$/.test(
                        model.getValueInRange(new Range(lineNumber, column - 3, lineNumber, column)),
                    );
                    paddingRight = model.getValueInRange(new Range(lineNumber, column, lineNumber, column + 1)) === '{';
                    edits.push({
                        range: new Range(lineNumber, column, lineNumber, column),
                        text: `${paddingLeft ? ' ' : ''}${label}${paddingRight ? ' ' : ''}`,
                    });
                    break;
                }
            }
            if (!label) continue;
            const hint: languages.InlayHint = {
                position: { lineNumber, column },
                label,
                kind,
                paddingLeft,
                paddingRight,
                textEdits: edits,
            };
            hints.push(hint);
        }
        return {
            hints,
            dispose: () => {
                // Cleanup if necessary
            },
        };
    }
}
export const inlayHintsProvider = new InlayHintsProvider();
languages.registerInlayHintsProvider('mirascript', inlayHintsProvider);
