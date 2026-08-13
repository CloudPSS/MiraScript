import type * as monaco from '@private/monaco-editor/baseapi';
import { vscode } from '#loader';

/**
 * Converts a Monaco Editor IMarkdownString to a VS Code MarkdownString.
 */
export function toMarkdownString<const T extends monaco.IMarkdownString | string | undefined = monaco.IMarkdownString>(
    markdown: T,
): T extends monaco.IMarkdownString ? vscode.MarkdownString : T {
    if (markdown == null) return undefined as never;
    if (typeof markdown == 'string') return markdown as never;
    if (markdown instanceof vscode.MarkdownString) return markdown as never;
    const result = new vscode.MarkdownString(markdown.value);
    if (markdown.isTrusted !== undefined) {
        result.isTrusted = markdown.isTrusted;
    }
    if (markdown.supportThemeIcons !== undefined) {
        result.supportThemeIcons = markdown.supportThemeIcons;
    }
    return result as never;
}
