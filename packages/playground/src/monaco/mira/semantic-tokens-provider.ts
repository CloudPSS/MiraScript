import { type CancellationToken, type editor, Emitter, type IEvent, languages } from 'monaco-editor';

/** @inheritdoc */
class DocumentSemanticTokensProvider implements languages.DocumentSemanticTokensProvider {
    private readonly _onDidChange = new Emitter<void>();
    /** @inheritdoc */
    get onDidChange(): IEvent<void> {
        return this._onDidChange.event;
    }
    /** @inheritdoc */
    getLegend(): languages.SemanticTokensLegend {
        return {
            tokenTypes: ['parameter', 'keyword'],
            tokenModifiers: ['xx'],
        };
    }
    /** @inheritdoc */
    provideDocumentSemanticTokens(
        model: editor.ITextModel,
        lastResultId: string | null,
        token: CancellationToken,
    ): languages.ProviderResult<languages.SemanticTokens | languages.SemanticTokensEdits> {
        return {
            resultId: 'x',
            data: new Uint32Array([1, 0, 2, 0, 1, 0, 3, 2, 1, 0]),
        };
    }
    /** @inheritdoc */
    releaseDocumentSemanticTokens(resultId: string | undefined): void {
        //
    }
}

languages.registerDocumentSemanticTokensProvider('mirascript', new DocumentSemanticTokensProvider());
