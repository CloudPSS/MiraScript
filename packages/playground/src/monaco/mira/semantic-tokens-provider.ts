import { type CancellationToken, type editor, languages } from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { DiagnosticCode } from 'mira-wasm';

/** @inheritdoc */
class DocumentSemanticTokensProvider extends Provider implements languages.DocumentSemanticTokensProvider {
    /** @inheritdoc */
    getLegend(): languages.SemanticTokensLegend {
        return {
            tokenTypes: ['entity.name.function', 'variable.other.constant', 'support.type.property-name'],
            tokenModifiers: [],
        };
    }
    /** @inheritdoc */
    async provideDocumentSemanticTokens(
        model: editor.ITextModel,
        lastResultId: string | null,
        token: CancellationToken,
    ): Promise<languages.SemanticTokens | languages.SemanticTokensEdits | null | undefined> {
        const resultId = `${model.uri.toString()}?${model.getVersionId()}`;
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) {
            return undefined;
        }

        // data 长度是 5 的倍数
        // [diffRow, diffCol, length, tokenType(index), tokenModifiers(bit field)]
        const data = [];
        for (const diagnostic of compiled.diagnostics) {
            let tokenType = -1;
            // eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
            switch (diagnostic.code) {
                case DiagnosticCode.GlobalFunction:
                case DiagnosticCode.LocalFunction: {
                    tokenType = 0;
                    break;
                }
                case DiagnosticCode.ParameterImmutable:
                case DiagnosticCode.ParameterImmutableRest:
                case DiagnosticCode.LocalImmutable: {
                    tokenType = 1;
                    break;
                }
                case DiagnosticCode.RecordFieldIdName: {
                    tokenType = 1;
                    break;
                }
            }
            if (tokenType < 0) continue;
            const { startLineNumber, startColumn, endColumn } = diagnostic;
            const length = endColumn - startColumn;
            data.push({
                row: startLineNumber - 1, // 从 0 开始
                col: startColumn - 1,
                length,
                tokenType,
                tokenModifiers: 0,
            });
        }
        data.sort((a, b) => {
            if (a.row !== b.row) {
                return a.row - b.row;
            }
            return a.col - b.col;
        });
        const bin = new Uint32Array(data.length * 5);
        for (let i = 0; i < data.length; i++) {
            const current = data[i]!;
            let { row, col } = current;
            if (i > 0) {
                const prev = data[i - 1]!;
                row -= prev.row;
                if (row === 0) {
                    col -= prev.col;
                }
            }
            bin.set([row, col, current.length, current.tokenType, current.tokenModifiers], i * 5);
        }
        return {
            resultId,
            data: bin,
        };
    }
    /** @inheritdoc */
    releaseDocumentSemanticTokens(resultId: string | undefined): void {
        //
    }
}

export const documentSemanticTokensProvider = new DocumentSemanticTokensProvider();
languages.registerDocumentSemanticTokensProvider('mirascript', documentSemanticTokensProvider);
