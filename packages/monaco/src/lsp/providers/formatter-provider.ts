import {
    type languages,
    type editor,
    type IRange,
    Range,
    type CancellationToken,
    type Position,
} from '../../monaco-api.js';
import { Provider } from './base.js';

/** 格式化 */
export class FormatterProvider
    extends Provider
    implements
        languages.DocumentFormattingEditProvider,
        languages.DocumentRangeFormattingEditProvider,
        languages.OnTypeFormattingEditProvider
{
    /** @inheritdoc */
    private async format(
        model: editor.ITextModel,
        ranges: readonly IRange[],
        options: languages.FormattingOptions,
        hint: 'full' | 'range' | 'expression' | 'statement',
        token: CancellationToken,
    ): Promise<languages.TextEdit[]> {
        if (hint !== 'full') return [];
        const compiled = await this.getCompileResult(model);
        if (!compiled?.result.formatted) return [];
        return [
            {
                range: model.getFullModelRange(),
                text: compiled.result.formatted,
            },
        ];
    }
    /** @inheritdoc */
    provideDocumentFormattingEdits(
        model: editor.ITextModel,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return this.format(model, [], options, 'full', token);
    }
    /** @inheritdoc */
    provideDocumentRangeFormattingEdits(
        model: editor.ITextModel,
        range: Range,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return this.format(model, [range], options, 'range', token);
    }
    /** @inheritdoc */
    provideDocumentRangesFormattingEdits(
        model: editor.ITextModel,
        ranges: Range[],
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return this.format(model, ranges, options, 'range', token);
    }
    /** @inheritdoc */
    readonly autoFormatTriggerCharacters = [';', '}', ']', ')'];
    /** @inheritdoc */
    provideOnTypeFormattingEdits(
        model: editor.ITextModel,
        position: Position,
        ch: string,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return this.format(
            model,
            [Range.fromPositions(position, position)],
            options,
            ch === ';' ? 'statement' : 'expression',
            token,
        );
    }
}
