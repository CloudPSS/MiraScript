import { isVmFunction, isVmModule } from '@mirascript/mirascript';
import { DiagnosticCode } from '@mirascript/wasm';
import { Range, type CancellationToken, type editor, type languages } from '../../monaco-api.js';
import { Provider } from './base.js';

enum TokenType {
    VARIABLE,
    VARIABLE_MUTABLE,
    CONSTANT,
    GLOBAL,
    FUNCTION,
    MODULE,
    PROPERTY,
    KEYWORD_CONTROL,
    PARAM,
    PARAM_MUTABLE,
}

const TOKEN_TYPES: Record<TokenType, string> = {
    [TokenType.VARIABLE]: 'variable.other.constant',
    [TokenType.VARIABLE_MUTABLE]: 'variable',
    [TokenType.CONSTANT]: 'variable.other.constant',
    [TokenType.GLOBAL]: 'variable',
    [TokenType.FUNCTION]: 'entity.name.function',
    [TokenType.MODULE]: 'entity.name.namespace',
    [TokenType.PROPERTY]: 'support.type.property-name',
    [TokenType.KEYWORD_CONTROL]: 'keyword.control',
    [TokenType.PARAM]: 'variable.other.constant.emphasis',
    [TokenType.PARAM_MUTABLE]: 'variable.emphasis',
};

/** @inheritdoc */
export class DocumentSemanticTokensProvider extends Provider implements languages.DocumentSemanticTokensProvider {
    /** @inheritdoc */
    getLegend(): languages.SemanticTokensLegend {
        const tokenTypes = Object.values(TOKEN_TYPES);
        return {
            tokenTypes,
            tokenModifiers: tokenTypes.map((_) => ''),
        };
    }
    /** @inheritdoc */
    async provideDocumentSemanticTokens(
        model: editor.ITextModel,
        lastResultId: string | null,
        token: CancellationToken,
    ): Promise<languages.SemanticTokens | null | undefined> {
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

            let tokenType: TokenType | -1 = -1;
            let onlyReferences = false;
            // eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
            switch (code) {
                case DiagnosticCode.GlobalVariable: {
                    const id = model.getValueInRange(range);
                    if (id.startsWith('@')) {
                        tokenType = TokenType.CONSTANT;
                    } else if (isVmFunction(globals.get(id))) {
                        tokenType = TokenType.FUNCTION;
                    } else if (isVmModule(globals.get(id))) {
                        tokenType = TokenType.MODULE;
                    } else {
                        tokenType = TokenType.GLOBAL;
                    }
                    break;
                }
                case DiagnosticCode.LocalFunction: {
                    tokenType = TokenType.FUNCTION;
                    break;
                }
                case DiagnosticCode.ParameterMutable:
                case DiagnosticCode.ParameterSubPatternMutable:
                case DiagnosticCode.ParameterMutableRest: {
                    tokenType = TokenType.PARAM_MUTABLE;
                    break;
                }
                case DiagnosticCode.LocalMutable: {
                    tokenType = TokenType.VARIABLE_MUTABLE;
                    break;
                }
                case DiagnosticCode.ParameterImmutable:
                case DiagnosticCode.ParameterSubPatternImmutable:
                case DiagnosticCode.ParameterImmutableRest: {
                    tokenType = TokenType.PARAM;
                    break;
                }
                case DiagnosticCode.LocalImmutable: {
                    tokenType = TokenType.VARIABLE;
                    break;
                }
                case DiagnosticCode.LocalConst: {
                    tokenType = TokenType.CONSTANT;
                    break;
                }
                case DiagnosticCode.RecordFieldIdName: {
                    tokenType = TokenType.PROPERTY;
                    break;
                }
                case DiagnosticCode.ForExpression: {
                    tokenType = TokenType.KEYWORD_CONTROL; // 标记为控制流关键字
                    onlyReferences = true; // for 表达式本身不需要标记
                    break;
                }
            }
            if (tokenType < 0) continue;
            const { startLineNumber, startColumn, endColumn } = range;
            const length = endColumn - startColumn;
            if (!onlyReferences) {
                data.push({
                    row: startLineNumber - 1, // 从 0 开始
                    col: startColumn - 1,
                    length,
                    tokenType,
                    tokenModifiers: 1 << tokenType,
                });
            }
            for (const ref of references) {
                data.push({
                    row: ref.range.startLineNumber - 1,
                    col: ref.range.startColumn - 1,
                    length: ref.range.endColumn - ref.range.startColumn,
                    tokenType,
                    tokenModifiers: 1 << tokenType,
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
