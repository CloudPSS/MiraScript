import { constants } from 'mirascript/subtle';

export const {
    REG_IDENTIFIER,
    REG_NUMBER,
    REG_HEX,
    REG_OCT,
    REG_BIN,
    REG_ORDINAL,
    constantKeywords,
    controlKeywords,
    keywords,
    numericKeywords,
    reservedKeywords,
} = constants;

export const REG_WHITESPACE = /[ \t\v\f\r\n]/u;

// Special characters
export const DOC_HEADER = ' 𝙼𝚒𝚛𝚊𝚂𝚌𝚛𝚒𝚙𝚝 𝙶𝚕𝚘𝚋𝚊𝚕 𝙳𝚎𝚏𝚒𝚗𝚒𝚝𝚒𝚘𝚗𝚜 ';

/** 基础语言服务支持的最大插值字符串 `$` 数量 */
export const MAX_VERBATIM_LENGTH = 16;
