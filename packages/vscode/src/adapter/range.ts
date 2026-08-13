import * as monaco from '@private/monaco-editor/baseapi';
import { vscode } from '#loader';

/**
 * Converts a Monaco Editor range to a VS Code Range.
 */
export function toRange<T extends monaco.IRange | null | undefined>(
    range: T,
): T extends monaco.IRange ? vscode.Range : T {
    if (range == null) return range as never;
    if (range instanceof vscode.Range) return range as never;
    return new vscode.Range(
        range.startLineNumber - 1,
        range.startColumn - 1,
        range.endLineNumber - 1,
        range.endColumn - 1,
    ) as never;
}

/**
 * Converts a VS Code Range to a Monaco Editor range.
 */
export function fromRange(range: vscode.Range): monaco.Range {
    if (range instanceof monaco.Range) return range;
    return new monaco.Range(
        range.start.line + 1,
        range.start.character + 1,
        range.end.line + 1,
        range.end.character + 1,
    );
}
