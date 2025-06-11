export const REG_IDENTIFIER = /(?:_+|@+|\$+|\p{XID_Start})\p{XID_Continue}*/u;
export const REG_NUMBER = /(?<!\.[ \t\v\f\r\n]*)\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/u;
export const REG_HEX = /0[xX][a-fA-F0-9_]*[a-fA-F0-9]/u;
export const REG_OCT = /0[oO][0-7_]*[0-7]/u;
export const REG_BIN = /0[bB][01_]*[01]/u;
export const REG_ORDINAL =
    /(?:0|[1-9]\d{0,8}|1\d{9}|20\d{8}|21[0-3]\d{7}|214[0-6]\d{6}|2147[0-3]\d{5}|21474[0-7]\d{4}|214748[0-2]\d{3}|2147483[0-5]\d{2}|21474836[0-3]\d|214748364[0-7])/u;

export { keywords, constantKeywords, numericKeywords, controlKeywords, reservedKeywords } from '../compiler/utils.js';
