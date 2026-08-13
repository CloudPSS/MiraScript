import type { languages as monacoLanguages } from '@private/monaco-editor';
import type { vscode } from '#loader';

/** Converts a Monaco Editor Command to a VS Code Command */
export function toCommand<const T extends monacoLanguages.Command | undefined = monacoLanguages.Command>(
    command: T,
): T extends monacoLanguages.Command ? vscode.Command : undefined {
    if (command == null) return undefined as never;
    if ('command' in command && 'title' in command) return command as never;
    return {
        title: command.title,
        command: command.id,
        arguments: command.arguments,
        tooltip: command.tooltip,
    } as never;
}
