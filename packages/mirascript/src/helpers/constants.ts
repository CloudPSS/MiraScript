export const REG_IDENTIFIER = /(?:_+|@+|\$+|\p{XID_Start})\p{XID_Continue}*/u;
export const REG_ORDINAL =
    /(?:214748364[0-7]|21474836[0-3]\d|2147483[0-5]\d{2}|214748[0-2]\d{3}|21474[0-7]\d{4}|2147[0-3]\d{5}|214[0-6]\d{6}|21[0-3]\d{7}|20\d{8}|1\d{9}|[1-9]\d{0,8}|0)/;
export const REG_WHITESPACE = /[ \t\v\f\r\n]/u;
export const REG_HEX = /0[xX][a-fA-F0-9_]*[a-fA-F0-9]/;
export const REG_OCT = /0[oO][0-7_]*[0-7]/;
export const REG_BIN = /0[bB][01_]*[01]/;
export const REG_NUMBER = /\d[\d_]*(?:\.[\d_]+)?(?:[eE][+-]?[\d_]*\d)?/u;
