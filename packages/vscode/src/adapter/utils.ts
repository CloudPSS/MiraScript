import {
    type IRange,
    type IPosition,
    Position as MonacoPosition,
    Range as MonacoRange,
    type IMarkdownString,
    type Uri as MonacoUri,
} from '@private/monaco-editor/baseapi';
import type { languages as monacoLanguages } from '@private/monaco-editor';
import {
    Range as VsRange,
    Position as VsPosition,
    TextEdit as VsTextEdit,
    MarkdownString as VsMarkdownString,
    Location as VsLocation,
    Uri as VsUri,
    type CompletionItem as VsCompletionItem,
    type CompletionItemKind,
    SnippetString,
} from 'vscode';

export enum CompletionItemInsertTextRule {
    None = 0,
    /**
     * Adjust whitespace/indentation of multiline insert texts to
     * match the current line indentation.
     */
    KeepWhitespace = 1,
    /**
     * `insertText` is a snippet.
     */
    InsertAsSnippet = 4,
}

/**
 * Converts a Monaco Editor range to a VS Code Range.
 */
export function toRange<T extends IRange | null | undefined>(range: T): T extends IRange ? VsRange : T {
    if (range == null) return range as never;
    if (range instanceof VsRange) return range as never;
    return new VsRange(
        range.startLineNumber - 1,
        range.startColumn - 1,
        range.endLineNumber - 1,
        range.endColumn - 1,
    ) as never;
}

/**
 * Converts a VS Code Range to a Monaco Editor range.
 */
export function fromRange(range: VsRange): MonacoRange {
    if (range instanceof MonacoRange) return range;
    return new MonacoRange(
        range.start.line + 1,
        range.start.character + 1,
        range.end.line + 1,
        range.end.character + 1,
    );
}

/**
 * Converts a Monaco Editor position to a VS Code Position.
 */
export function toPosition(position: IPosition): VsPosition {
    if (position instanceof VsPosition) return position;
    return new VsPosition(position.lineNumber - 1, position.column - 1);
}

/**
 * Converts a VS Code Position to a Monaco Editor position.
 */
export function fromPosition(position: VsPosition): MonacoPosition {
    if (position instanceof MonacoPosition) return position;
    return new MonacoPosition(position.line + 1, position.character + 1);
}
/**
 * Converts a Monaco Editor TextEdit to a VS Code TextEdit.
 */
export function toTextEdit(edit: { range: IRange; text: string | null }): VsTextEdit {
    if (edit instanceof VsTextEdit) return edit;
    if (!edit.text) {
        return VsTextEdit.delete(toRange(edit.range));
    }
    return VsTextEdit.replace(toRange(edit.range), edit.text);
}

/**
 * Converts a Monaco Editor IMarkdownString to a VS Code MarkdownString.
 */
export function toMarkdownString<const T extends IMarkdownString | string | undefined = IMarkdownString>(
    markdown: T,
): T extends IMarkdownString ? VsMarkdownString : T {
    if (markdown == null) return undefined as never;
    if (typeof markdown == 'string') return markdown as never;
    if (markdown instanceof VsMarkdownString) return markdown as never;
    const result = new VsMarkdownString(markdown.value);
    if (markdown.isTrusted !== undefined) {
        result.isTrusted = markdown.isTrusted;
    }
    if (markdown.supportThemeIcons !== undefined) {
        result.supportThemeIcons = markdown.supportThemeIcons;
    }
    return result as never;
}

/**
 * Converts a Monaco Editor Location to a VS Code Location.
 */
export function toLocation(location: { uri: MonacoUri; range: IRange } | undefined): VsLocation | undefined {
    if (location == null) return undefined;
    if (location instanceof VsLocation) return location;
    return new VsLocation(VsUri.parse(location.uri.toString()), toRange(location.range));
}
/**
 * Convert to VS Code CompletionItem
 */
export function toCompletionItem(
    item: Omit<monacoLanguages.CompletionItem, 'kind'> & { kind: number },
): VsCompletionItem & { original: monacoLanguages.CompletionItem } {
    const range =
        'insert' in item.range
            ? {
                  inserting: toRange(item.range.insert),
                  replacing: toRange(item.range.replace),
              }
            : toRange(item.range);
    const ci: VsCompletionItem & { original: monacoLanguages.CompletionItem } = {
        label: item.label,
        kind: item.kind as CompletionItemKind,
        tags: item.tags,
        detail: item.detail,
        documentation: toMarkdownString(item.documentation),
        sortText: item.sortText,
        filterText: item.filterText,
        preselect: item.preselect,
        insertText:
            item.insertTextRules === CompletionItemInsertTextRule.InsertAsSnippet
                ? new SnippetString(item.insertText)
                : item.insertText,
        range,
        commitCharacters: item.commitCharacters,
        additionalTextEdits: item.additionalTextEdits?.map(toTextEdit),
        command: undefined,
        original: item,
    };
    return ci;
}
