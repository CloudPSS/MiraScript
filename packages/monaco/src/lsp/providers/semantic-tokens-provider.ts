import { isVmFunction, isVmModule } from '@mirascript/mirascript';
import { DiagnosticCode } from '@mirascript/wasm';
import { Range, type CancellationToken, type editor, type languages } from '../../monaco-api.js';
import { Provider } from './base.js';
import { ParameterDefinitionType } from '../compile-result.js';

/** @inheritdoc */
export class DocumentSemanticTokensProvider extends Provider implements languages.DocumentSemanticTokensProvider {
    /** @inheritdoc */
    getLegend(): languages.SemanticTokensLegend {
        return {
            tokenTypes: [
                '',
                'variable',
                'entity.name.function',
                'variable.other.constant',
                'support.type.property-name',
                'keyword.control',
            ],
            tokenModifiers: ['strong', 'emphasis', 'underline', 'strikethrough'],
        };
    }
    /** @inheritdoc */
    async provideDocumentSemanticTokens(
        model: editor.ITextModel,
        lastResultId: string | null,
        token: CancellationToken,
    ): Promise<languages.SemanticTokens | languages.SemanticTokensEdits | null | undefined> {
        const resultId = `${model.uri.toString()}?${model.getVersionId()}`;
        const compiled = await this.getCompileResult(model);
        if (!compiled) {
            return undefined;
        }
        const globals = await this.getContext(model);

        // data 长度是 5 的倍数
        // [diffRow, diffCol, length, tokenType(index), tokenModifiers(bit field)]
        const data = [];
        for (const { code, range, references } of compiled.tags) {
            if (Range.isEmpty(range)) continue;

            let tokenType = -1;
            let tokenModifiers = 0;
            let onlyReferences = false;
            // eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
            switch (code) {
                case DiagnosticCode.GlobalVariable: {
                    const id = model.getValueInRange(range);
                    if (id.startsWith('@')) {
                        tokenType = 3;
                    } else if (isVmFunction(globals[id])) {
                        tokenType = 2;
                    } else if (isVmModule(globals[id])) {
                        tokenModifiers += 1 << 1;
                        tokenType = 3;
                    } else {
                        tokenType = 1;
                    }
                    // tokenModifiers += 1 << 2; // 全局变量添加下划线
                    break;
                }
                case DiagnosticCode.ParameterMutable:
                case DiagnosticCode.ParameterSubPatternMutable:
                case DiagnosticCode.ParameterMutableRest:
                case DiagnosticCode.LocalMutable: {
                    tokenType = 1;
                    break;
                }
                case DiagnosticCode.LocalFunction: {
                    tokenType = 2;
                    break;
                }
                case DiagnosticCode.ParameterImmutable:
                case DiagnosticCode.ParameterSubPatternImmutable:
                case DiagnosticCode.ParameterImmutableRest:
                case DiagnosticCode.LocalImmutable:
                case DiagnosticCode.LocalConst: {
                    tokenType = 3;
                    break;
                }
                case DiagnosticCode.RecordFieldIdName: {
                    tokenType = 4;
                    break;
                }
                case DiagnosticCode.ForExpression: {
                    tokenType = 5; // 标记为控制流关键字
                    onlyReferences = true; // for 表达式本身不需要标记
                    break;
                }
            }
            if (tokenType < 0) continue;
            if (ParameterDefinitionType.includes(code as ParameterDefinitionType)) {
                tokenModifiers |= 1 << 1;
            }
            const { startLineNumber, startColumn, endColumn } = range;
            const length = endColumn - startColumn;
            if (!onlyReferences) {
                data.push({
                    row: startLineNumber - 1, // 从 0 开始
                    col: startColumn - 1,
                    length,
                    tokenType,
                    tokenModifiers,
                });
            }
            for (const ref of references) {
                data.push({
                    row: ref.range.startLineNumber - 1,
                    col: ref.range.startColumn - 1,
                    length: ref.range.endColumn - ref.range.startColumn,
                    tokenType,
                    tokenModifiers,
                });
            }
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
