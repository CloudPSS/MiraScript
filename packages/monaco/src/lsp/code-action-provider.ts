import { type editor, languages, type Range, type CancellationToken } from '@private/monaco-editor';
import { Provider } from './worker-helper.js';

/**
 * 代码操作
 */
export class CodeActionProvider extends Provider implements languages.CodeActionProvider {
    /** @inheritdoc */
    provideCodeActions(
        model: editor.ITextModel,
        range: Range,
        context: languages.CodeActionContext,
        token: CancellationToken,
    ): languages.ProviderResult<languages.CodeActionList> {
        return {
            actions: [],
            dispose: () => void 0,
        };
    }

    /** @inheritdoc */
    resolveCodeAction?(
        codeAction: languages.CodeAction,
        token: CancellationToken,
    ): languages.ProviderResult<languages.CodeAction> {
        throw new Error('Method not implemented.');
    }
}
