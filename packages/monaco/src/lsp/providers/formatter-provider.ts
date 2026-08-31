import { type languages, type editor, type IRange, Range, type CancellationToken } from '../../monaco-api.js';
import { parseDiagnostics } from '@mirascript/mirascript/subtle';
import { formatModel } from '../worker-helper.js';
import { Provider } from './base.js';

const DEFAULT_PRINT_WIDTH = 80;

/** 请求格式化并将 UTF-16 偏移编辑转换为 Monaco 编辑。 */
async function format(
    model: editor.ITextModel,
    ranges: readonly IRange[] | undefined,
    options: languages.FormattingOptions | undefined,
    token: CancellationToken | undefined,
): Promise<languages.TextEdit[] | null> {
    if (model.uri.scheme === 'mirascript' || token?.isCancellationRequested) return [];
    if (ranges?.length === 0) return [];
    const version = model.getVersionId();
    const offsetRanges = ranges?.map((range) => ({
        start: model.getOffsetAt(Range.getStartPosition(range)),
        end: model.getOffsetAt(Range.getEndPosition(range)),
    }));
    try {
        const result = await formatModel(model, offsetRanges, {
            tabSize: options?.tabSize ?? 2,
            insertSpaces: options?.insertSpaces ?? true,
            printWidth: DEFAULT_PRINT_WIDTH,
        });
        if (model.isDisposed() || token?.isCancellationRequested || model.getVersionId() !== version) return [];
        if (ranges == null && parseDiagnostics(model.getValue(), result.diagnostics).errors.length > 0) return null;
        return result.edits.map((edit) => ({
            range: Range.fromPositions(model.getPositionAt(edit.start), model.getPositionAt(edit.end)),
            text: edit.text,
        }));
    } catch (error) {
        // eslint-disable-next-line no-console
        console.error('MiraScript formatting failed:', error);
        return null;
    }
}

/** MiraScript 整文及选区格式化。 */
export class FormatterProvider
    extends Provider
    implements languages.DocumentFormattingEditProvider, languages.DocumentRangeFormattingEditProvider
{
    /** 返回完整格式化文本；源码已规范时返回原文本。 */
    static async format(model: editor.ITextModel, options?: languages.FormattingOptions): Promise<string | null> {
        const edits = await format(model, undefined, options, undefined);
        if (edits == null) return null;
        if (edits.length === 0) return model.getValue();
        let output = model.getValue();
        const offsetEdits = edits
            .map((edit) => ({
                start: model.getOffsetAt(Range.getStartPosition(edit.range)),
                end: model.getOffsetAt(Range.getEndPosition(edit.range)),
                text: edit.text,
            }))
            .toSorted((left, right) => right.start - left.start);
        for (const edit of offsetEdits) {
            output = output.slice(0, edit.start) + edit.text + output.slice(edit.end);
        }
        return output;
    }

    /** 提供整篇文档格式化编辑。 */
    provideDocumentFormattingEdits(
        model: editor.ITextModel,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return format(model, undefined, options, token).then((edits) => edits ?? []);
    }

    /** 提供单个选区格式化编辑。 */
    provideDocumentRangeFormattingEdits(
        model: editor.ITextModel,
        range: Range,
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return format(model, [range], options, token).then((edits) => edits ?? []);
    }

    /** 提供多个选区格式化编辑。 */
    provideDocumentRangesFormattingEdits(
        model: editor.ITextModel,
        ranges: Range[],
        options: languages.FormattingOptions,
        token: CancellationToken,
    ): languages.ProviderResult<languages.TextEdit[]> {
        return format(model, ranges, options, token).then((edits) => edits ?? []);
    }
}
