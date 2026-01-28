/**
 * import 类型声明
 * @import {type editor, type IPosition} from '../monaco-api.js';
 * @typedef {'other' | 'comment' | 'string' | 'regex'} TokenType 标准 token 类型字符串
 */

/**
 * 解析标准 token 类型
 * @param {number } tokenType 标准 token 类型
 * @returns {TokenType } 标准 token 类型字符串
 */
function fromStandardTokenType(tokenType) {
    switch (tokenType) {
        case 1:
            return 'comment';
        case 2:
            return 'string';
        case 3:
            return 'regex';
        default:
            return 'other';
    }
}

/**
 * 获取 token
 * @param {editor.ITextModel} model 文本模型
 * @param {IPosition} position 位置
 * @returns {{type: TokenType, text: string | null, startColumn: number, endColumn: number} | undefined} token 字符串或未定义
 */
export function tokenAt(model, position) {
    try {
        const lineTokens = model.tokenization.getLineTokens(position.lineNumber);
        const tokenIndex = lineTokens.findTokenIndexAtOffset(position.column - 1);
        if (tokenIndex < 0) return undefined;

        return {
            type: fromStandardTokenType(tryGetTokenProp('getStandardTokenType', 0)),
            text: tryGetTokenProp('getTokenText', null),
            startColumn: tryGetTokenProp('getStartOffset', 0) + 1,
            endColumn: tryGetTokenProp('getEndOffset', 0) + 1,
        };

        /** @type {<T>(method: string, fallback: T) => T} */
        function tryGetTokenProp(method, fallback) {
            try {
                return lineTokens[method](tokenIndex);
            } catch {
                return fallback;
            }
        }
    } catch {
        return undefined;
    }
}
