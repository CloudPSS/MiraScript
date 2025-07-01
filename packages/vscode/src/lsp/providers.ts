import { Disposable, languages } from 'vscode';
import { FormatterProvider } from '@mirascript/monaco/lsp';
import { ModelAdapter } from '../adapter/model.js';
import { toTextEdit } from '../adapter/utils.js';

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
        this.registerProviders();
    }

    /** 注册 Providers */
    private registerProviders(): void {
        const selector = ['mirascript', 'mirascript-template'];
        const formatter = new FormatterProvider();
        this.disposables.push(
            languages.registerDocumentFormattingEditProvider(selector, {
                provideDocumentFormattingEdits: async (document, options, token) => {
                    const result = await formatter.provideDocumentFormattingEdits(
                        new ModelAdapter(document),
                        options,
                        token,
                    );
                    if (!result) return result;
                    return result.map(toTextEdit);
                },
            }),
        );
    }
}
