export { REG_IDENTIFIER, REG_HEX, REG_OCT, REG_BIN, REG_ORDINAL, REG_WHITESPACE } from '../../helpers/constants.js';

export const REG_NUMBER = /(?<!\.\s*)\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/u;
export const DOC_HEADER = ' MiraScript Global Definitions ';
