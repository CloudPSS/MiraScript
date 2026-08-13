import * as monaco from '@private/monaco-editor/baseapi';
import { vscode } from '#loader';

/**
 * Converts a Monaco Editor position to a VS Code Position.
 */
export function toPosition(position: monaco.IPosition): vscode.Position {
    if (position instanceof vscode.Position) return position;
    return new vscode.Position(position.lineNumber - 1, position.column - 1);
}

/**
 * Converts a VS Code Position to a Monaco Editor position.
 */
export function fromPosition(position: vscode.Position): monaco.Position {
    if (position instanceof monaco.Position) return position;
    return new monaco.Position(position.line + 1, position.character + 1);
}
