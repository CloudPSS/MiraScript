import type { languages as monacoLanguages } from '@private/monaco-editor';
import { vscode } from '#loader';
import { toRange } from './range.js';
import { toUri } from './uri.js';

/**
 * Converts a Monaco Editor Location to a VS Code Location.
 */
export function toLocation<const T extends monacoLanguages.Location | undefined = monacoLanguages.Location>(
    location: T,
): T extends monacoLanguages.Location ? vscode.Location : undefined {
    if (location == null) return undefined as never;
    if (location instanceof vscode.Location) return location as never;
    return new vscode.Location(toUri(location.uri), toRange(location.range)) as never;
}
