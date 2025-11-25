import { languages, Range, type CancellationToken, type editor, type IEvent } from '../../monaco-api.js';
import { Provider } from './base.js';
import { DiagnosticCode } from '@mirascript/bindings/wasm';

/** @inheritdoc */
export class InlayHintsProvider extends Provider implements languages.InlayHintsProvider {
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
        const compiled = await this.getCompileResult(model);
        if (!compiled) {
            return undefined;
        }
        const hints: languages.InlayHint[] = [];
        for (const tag of compiled.tags) {
            if (!range.containsRange(tag.range)) {
                continue;
            }
            let lineNumber = 0;
            let column = 0;
            let label = '';
            const kind = languages.InlayHintKind.Parameter;
            let paddingLeft = true;
            let paddingRight = false;
            const edits: languages.TextEdit[] = [];
            // eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
            switch (tag.code) {
                case DiagnosticCode.ParameterIt: {
                    if (!tag.references.length) {
                        // 没有引用，隐藏该隐式参数
                        continue;
                    }
                    lineNumber = tag.range.endLineNumber;
                    column = tag.range.endColumn;
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
                    const index = tag.code - DiagnosticCode.UnnamedRecordField0;
                    if (index > 9) break;
                    lineNumber = tag.range.startLineNumber;
                    column = tag.range.startColumn;
                    label = `${index}:`;
                    paddingLeft = /[^\s(]/u.test(
                        model.getValueInRange(new Range(lineNumber, column - 1, lineNumber, column)),
                    );
                    paddingRight = true;
                    break;
                }
                case DiagnosticCode.OmitNamedRecordField: {
                    const ref = tag.references[0];
                    if (ref?.code !== DiagnosticCode.OmitNamedRecordFieldName) continue;
                    lineNumber = tag.range.startLineNumber;
                    column = tag.range.startColumn;
                    label = model.getValueInRange(ref.range);
                    paddingLeft = /[^\s(]/u.test(
                        model.getValueInRange(new Range(lineNumber, column - 1, lineNumber, column)),
                    );
                    paddingRight = false;

                    const insertRight = !/\s/u.test(
                        model.getValueInRange(
                            new Range(
                                tag.range.endLineNumber,
                                tag.range.endColumn,
                                tag.range.endLineNumber,
                                tag.range.endColumn + 1,
                            ),
                        ),
                    );
                    edits.push({
                        range: tag.range,
                        text: `${paddingLeft ? ' ' : ''}${label}${model.getValueInRange(tag.range)}${insertRight ? ' ' : ''}`,
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
