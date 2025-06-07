import { type editor, languages, type CancellationToken, type Position } from '@private/monaco-editor';
import { Provider } from './worker-helper';

/** @inheritdoc */
class SignatureHelpProvider extends Provider implements languages.SignatureHelpProvider {
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
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) return undefined;

        return undefined; // TODO: Implement signature help logic
    }
}

languages.registerSignatureHelpProvider('mirascript', new SignatureHelpProvider());
