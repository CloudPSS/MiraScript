export { REG_IDENTIFIER, REG_HEX, REG_OCT, REG_BIN, REG_ORDINAL, REG_WHITESPACE } from '../../helpers/constants.js';

export const REG_NUMBER = /(?<!\.\s*)\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/u;

// Special characters
export const DOC_HEADER = ' 𝙼𝚒𝚛𝚊𝚂𝚌𝚛𝚒𝚙𝚝 𝙶𝚕𝚘𝚋𝚊𝚕 𝙳𝚎𝚏𝚒𝚗𝚒𝚝𝚒𝚘𝚗𝚜 ';

export const MAX_VERBATIM_LENGTH = 16;
