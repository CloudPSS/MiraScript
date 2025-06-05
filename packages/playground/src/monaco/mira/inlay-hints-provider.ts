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
        for (let i = 0; i < compiled.diagnostics.length; i++) {
            const diagnostic = compiled.diagnostics[i]!;
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
                    paddingLeft = /\bfn$/u.test(
                        model.getValueInRange(new Range(lineNumber, column - 3, lineNumber, column)),
                    );
                    paddingRight = model.getValueInRange(new Range(lineNumber, column, lineNumber, column + 1)) === '{';
                    edits.push({
                        range: new Range(lineNumber, column, lineNumber, column),
                        text: `${paddingLeft ? ' ' : ''}${label}${paddingRight ? ' ' : ''}`,
                    });
                    break;
                }
                case DiagnosticCode.UnnamedRecordField0:
                case DiagnosticCode.UnnamedRecordField1:
                case DiagnosticCode.UnnamedRecordField2:
                case DiagnosticCode.UnnamedRecordField3:
                case DiagnosticCode.UnnamedRecordField4:
                case DiagnosticCode.UnnamedRecordField5:
                case DiagnosticCode.UnnamedRecordField6:
                case DiagnosticCode.UnnamedRecordField7:
                case DiagnosticCode.UnnamedRecordField8:
                case DiagnosticCode.UnnamedRecordField9:
                case DiagnosticCode.UnnamedRecordFieldN: {
                    const index = diagnostic.code - DiagnosticCode.UnnamedRecordField0;
                    if (index > 9) break;
                    lineNumber = diagnostic.startLineNumber;
                    column = diagnostic.startColumn;
                    label = `${index}:`;
                    paddingLeft = /[^\s(]/u.test(
                        model.getValueInRange(new Range(lineNumber, column - 1, lineNumber, column)),
                    );
                    paddingRight = true;
                    break;
                }
                case DiagnosticCode.OmitNamedRecordField: {
                    const next = compiled.diagnostics[i + 1];
                    if (next?.code !== DiagnosticCode.OmitNamedRecordFieldName) continue;
                    i++;
                    lineNumber = diagnostic.startLineNumber;
                    column = diagnostic.startColumn;
                    label = model.getValueInRange(next);
                    paddingLeft = /[^\s(]/u.test(
                        model.getValueInRange(new Range(lineNumber, column - 1, lineNumber, column)),
                    );
                    paddingRight = false;

                    const insertRight = !/\s/u.test(
                        model.getValueInRange(
                            new Range(
                                diagnostic.endLineNumber,
                                diagnostic.endColumn,
                                diagnostic.endLineNumber,
                                diagnostic.endColumn + 1,
                            ),
                        ),
                    );
                    edits.push({
                        range: diagnostic,
                        text: `${paddingLeft ? ' ' : ''}${label}${model.getValueInRange(diagnostic)}${insertRight ? ' ' : ''}`,
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
languages.registerInlayHintsProvider('mirascript', new InlayHintsProvider());
