export {
    KEYWORDS,
    CONSTANT_KEYWORDS,
    CONTROL_KEYWORDS,
    NUMERIC_KEYWORDS,
    RESERVED_KEYWORDS,
} from '@mirascript/constants';
import { constants, KEYWORDS } from '@mirascript/mirascript/subtle';

export const {
    REG_IDENTIFIER,
    REG_ORDINAL,
    REG_WHITESPACE,
    REG_BIN,
    REG_HEX,
    REG_OCT,
    REG_NUMBER,
    REG_IDENTIFIER_FULL,
    REG_ORDINAL_FULL,
} = constants;

/** 基础语言服务支持的最大插值字符串 `$` 数量 */
export const MAX_VERBATIM_LENGTH = 16;

const keywordsSet = new Set<string>(KEYWORDS);

/**
 * 判断是否为 MiraScript 关键字
 */
export function isKeyword(word: string): boolean {
    return keywordsSet.has(word);
}
