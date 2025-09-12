import { Disposable, languages, SemanticTokens } from 'vscode';
import { FormatterProvider, DocumentSemanticTokensProvider } from '@mirascript/monaco/lsp';
import { ModelAdapter } from '../adapter/model.js';
import { toTextEdit } from '../adapter/utils.js';
import { registerMonacoApi } from '@mirascript/monaco';
import * as monaco from '@private/monaco-editor/baseapi';

/**
 * Manages all language service providers.
 */
export class ProvidersManager extends Disposable {
    private readonly disposables: Disposable[] = [];

    constructor() {
        super(() => {
            for (const disposable of this.disposables) {
                disposable.dispose();
            }
        });
        registerMonacoApi(monaco);
        this.registerProviders();
    }

    /** 注册 Providers */
    private registerProviders(): void {
        const selector = ['mirascript', 'mirascript-template'];
        const formatterProvider = new FormatterProvider();
        const documentSemanticTokensProvider = new DocumentSemanticTokensProvider();
        this.disposables.push(
            languages.registerDocumentFormattingEditProvider(selector, {
                provideDocumentFormattingEdits: async (document, options, token) => {
                    const result = await formatterProvider.provideDocumentFormattingEdits(
                        new ModelAdapter(document),
                        options,
                        token,
                    );
                    if (!result) return result;
                    return result.map(toTextEdit);
                },
            }),
            languages.registerDocumentSemanticTokensProvider(
                selector,
                {
                    provideDocumentSemanticTokens: async (document, token) => {
                        const result = await documentSemanticTokensProvider.provideDocumentSemanticTokens(
                            new ModelAdapter(document),
                            null,
                            token,
                        );
                        if (!result) return null;
                        return new SemanticTokens(result.data);
                    },
                },
                {
                    tokenTypes: [
                        'variable',
                        'variable',
                        'variable',
                        'variable',
                        'function',
                        'namespace',
                        'property',
                        'keyword',
                        'parameter',
                        'parameter',
                    ],
                    tokenModifiers: ['readonly', '', 'readonly', '', '', 'readonly', '', 'controlFlow', 'readonly', ''],
                },
            ),
        );
    }
}
