import { type CancellationToken, type editor, Emitter, type IEvent, languages } from '@private/monaco-editor';

/** @inheritdoc */
class DocumentSemanticTokensProvider implements languages.DocumentSemanticTokensProvider {
    private readonly _onDidChange = new Emitter<void>();
    /** @inheritdoc */
    get onDidChange(): IEvent<void> {
        return this._onDidChange.event;
    }
    /** 触发 onDidChange */
    emitDidChange(): void {
        this._onDidChange.fire();
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
        // data 长度是 5 的倍数
        // [diffRow, diffCol, length, tokenType(index), tokenModifiers(bit field)]
        return {
            resultId: 'x',
            data: new Uint32Array([]),
        };
    }
    /** @inheritdoc */
    releaseDocumentSemanticTokens(resultId: string | undefined): void {
        //
    }
}

languages.registerDocumentSemanticTokensProvider('mirascript', new DocumentSemanticTokensProvider());
