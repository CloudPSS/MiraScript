import type { IRange, IPosition } from '@private/monaco-editor';
import { Range, Position, TextEdit } from 'vscode';

/**
 * Converts a Monaco Editor range to a VS Code Range.
 */
export function toRange(range: IRange): Range {
    if (range instanceof Range) return range;
    return new Range(range.startLineNumber - 1, range.startColumn - 1, range.endLineNumber - 1, range.endColumn - 1);
}

/**
 * Converts a Monaco Editor position to a VS Code Position.
 */
export function toPosition(position: IPosition): Position {
    if (position instanceof Position) return position;
    return new Position(position.lineNumber - 1, position.column - 1);
}

/**
 * Converts a Monaco Editor TextEdit to a VS Code TextEdit.
 */
export function toTextEdit(edit: { range: IRange; text: string }): TextEdit {
    if (edit instanceof TextEdit) return edit;
    return new TextEdit(toRange(edit.range), edit.text);
}
