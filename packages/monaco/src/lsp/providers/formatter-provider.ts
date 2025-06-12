import type { languages, editor, IRange, Range, CancellationToken, Position } from '@private/monaco-editor';
import { Provider } from './base.js';

/** @inheritdoc */
function format(
    model: editor.ITextModel,
    ranges: readonly IRange[],
    options: languages.FormattingOptions,
    hint: 'range' | 'expression' | 'statement' | 'block',
    token: CancellationToken,
): languages.TextEdit[] {
    // TODO:
    return [];
}

/** 格式化 */
export class FormatterProvider
    extends Provider
    implements
        languages.DocumentFormattingEditProvider,
        languages.DocumentRangeFormattingEditProvider,
        languages.OnTypeFormattingEditProvider
{
    /** @inheritdoc */
    provideDocumentFormattingEdits(
        model: editor.ITextModel,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return format(model, [model.getFullModelRange()], options, 'range', token);
    }
    /** @inheritdoc */
    provideDocumentRangeFormattingEdits(
        model: editor.ITextModel,
        range: Range,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return format(model, [range], options, 'range', token);
    }
    /** @inheritdoc */
    provideDocumentRangesFormattingEdits(
        model: editor.ITextModel,
        ranges: Range[],
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return format(model, ranges, options, 'range', token);
    }
    /** @inheritdoc */
    readonly autoFormatTriggerCharacters = [';', '}', ']', ')', '\n'];
    /** @inheritdoc */
    provideOnTypeFormattingEdits(
        model: editor.ITextModel,
        position: Position,
        ch: string,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return format(model, [this.monaco.Range.fromPositions(position, position)], options, 'expression', token);
    }
}
