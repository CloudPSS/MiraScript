import { type CancellationToken, editor, Emitter, type IEvent, languages, Uri } from 'monaco-editor';
import { compile_script, get_error_message } from 'mira-wasm';

const errorMessages = new Map<number, string | undefined>();
/** 获取错误消息 */
function getErrorMessage(code: number): string | undefined {
    if (code < 0 || code >= 65536) return undefined;
    if (errorMessages.has(code)) return errorMessages.get(code);
    const message = get_error_message(code);
    errorMessages.set(code, message);
    return message;
}

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
        const text = model.getValue();
        const version = model.getVersionId();
        const errors = compile_script(text);
        const markers: editor.IMarkerData[] = [];
        for (let i = 0; i < errors.length; i += 3) {
            const start = model.getPositionAt(errors[i]);
            const end = model.getPositionAt(errors[i + 1]);
            const error = errors[i + 2];
            const message = getErrorMessage(error) ?? 'Unknown error';
            markers.push({
                startLineNumber: start.lineNumber,
                startColumn: start.column,
                endLineNumber: end.lineNumber,
                endColumn: end.column,
                message,
                modelVersionId: version,
                severity: 8,
                source: 'mira',
                code: {
                    value: `${error}`,
                    target: Uri.parse(`https://mira.com/${error}`),
                },
            });
        }
        editor.setModelMarkers(model, 'mira', markers);
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
