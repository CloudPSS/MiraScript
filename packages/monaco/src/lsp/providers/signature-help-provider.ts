import type { editor, languages, CancellationToken, Position } from '@private/monaco-editor';
import { Provider } from './base.js';

/** @inheritdoc */
export class SignatureHelpProvider extends Provider implements languages.SignatureHelpProvider {
    /** @inheritdoc */
    readonly signatureHelpTriggerCharacters = ['(', ','];
    /** @inheritdoc */
    readonly signatureHelpRetriggerCharacters = [')'];
    /** @inheritdoc */
    async provideSignatureHelp(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
        context: languages.SignatureHelpContext,
    ): Promise<languages.SignatureHelpResult | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;

        return undefined; // TODO: Implement signature help logic
    }
}
