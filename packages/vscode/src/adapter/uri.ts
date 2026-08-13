import type * as monaco from '@private/monaco-editor/baseapi';
import { vscode } from '#loader';

/**
 * Converts a Monaco Editor Uri to a VS Code Uri.
 */
export function toUri<const T extends monaco.Uri | undefined = monaco.Uri>(
    uri: T,
): T extends monaco.Uri ? vscode.Uri : T {
    if (uri == null) return undefined as never;
    if (uri instanceof vscode.Uri) return uri as never;
    return vscode.Uri.parse(uri.toString()) as never;
}
