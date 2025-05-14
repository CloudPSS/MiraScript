import { languages, type editor, type IRange, Range, type CancellationToken } from '@private/monaco-editor';

/** @inheritdoc */
function format(
    model: editor.ITextModel,
    range: IRange,
    options: languages.FormattingOptions,
    hint: 'range' | 'expression' | 'statement' | 'block',
    token: CancellationToken,
): languages.TextEdit[] {
    // TODO:
    return [];
}

languages.registerDocumentRangeFormattingEditProvider('mirascript', {
    provideDocumentRangeFormattingEdits(model, range, options, token) {
        return format(model, range, options, 'range', token);
    },
});

languages.registerOnTypeFormattingEditProvider('mirascript', {
    autoFormatTriggerCharacters: [';', '}', '\n'],
    provideOnTypeFormattingEdits(model, position, ch, options, token) {
        const previous = ch === '\n' ? position.delta(-1) : model.modifyPosition(position, -1);
        const range = Range.fromPositions(previous, position);
        return format(model, range, options, ch === ';' ? 'statement' : ch === '}' ? 'block' : 'expression', token);
    },
});
