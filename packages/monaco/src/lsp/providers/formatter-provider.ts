import {
    type languages,
    type editor,
    type IRange,
    Range,
    type CancellationToken,
    type Position,
} from '../../monaco-api.js';
import { Provider } from './base.js';

/** 进行格式化 */
async function format(
    model: editor.ITextModel,
    ranges: readonly IRange[],
    options: languages.FormattingOptions | undefined,
    hint: 'full' | 'range' | 'expression' | 'statement',
    token: CancellationToken | undefined,
): Promise<languages.TextEdit[]> {
    if (hint !== 'full') return [];
    const compiled = await Provider.getCompileResult(model);
    if (compiled?.result.formatted == null) return [];
    return [
        {
            range: model.getFullModelRange(),
            text: compiled.result.formatted,
        },
    ];
}

/** 格式化 */
export class FormatterProvider
    extends Provider
    implements
        languages.DocumentFormattingEditProvider,
        languages.DocumentRangeFormattingEditProvider,
        languages.OnTypeFormattingEditProvider
{
    /** 进行完整格式化 */
    static async format(model: editor.ITextModel, options?: languages.FormattingOptions): Promise<string | null> {
        const formatted = await format(model, [], options, 'full', undefined);
        return formatted.length === 1 ? formatted[0]!.text : null;
    }
    /** @inheritdoc */
    provideDocumentFormattingEdits(
        model: editor.ITextModel,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return format(model, [], options, 'full', token);
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
    readonly autoFormatTriggerCharacters = [';', '}', ']', ')'];
    /** @inheritdoc */
    provideOnTypeFormattingEdits(
        model: editor.ITextModel,
        position: Position,
        ch: string,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return format(
            model,
            [Range.fromPositions(position, position)],
            options,
            ch === ';' ? 'statement' : 'expression',
            token,
        );
    }
}
