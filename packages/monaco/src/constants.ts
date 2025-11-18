import { constants, keywords } from '@mirascript/mirascript/subtle';
import { getModule } from '@mirascript/bindings/wasm';

export { keywords };

const wasm = () => getModule().wasm;

/** MiraScript 控制流关键字 */
export let controlKeywords = (): readonly string[] => {
    const kw = wasm().control_keywords();
    Object.freeze(kw);
    controlKeywords = () => kw;
    return kw;
};

/** MiraScript 数值字面量关键字 */
export let numericKeywords = (): readonly string[] => {
    const kw = wasm().numeric_keywords();
    Object.freeze(kw);
    numericKeywords = () => kw;
    return kw;
};

/** MiraScript 字面量关键字 */
export let constantKeywords = (): readonly string[] => {
    const kw = wasm().constant_keywords();
    Object.freeze(kw);
    constantKeywords = () => kw;
    return kw;
};

/** MiraScript 保留字关键字 */
export let reservedKeywords = (): readonly string[] => {
    const kw = wasm().reserved_keywords();
    Object.freeze(kw);
    reservedKeywords = () => kw;
    return kw;
};

export const { REG_IDENTIFIER, REG_ORDINAL } = constants;

export const REG_WHITESPACE = /[ \t\v\f\r\n]/u;
export const REG_HEX = /0[xX][a-fA-F0-9_]*[a-fA-F0-9]/;
export const REG_OCT = /0[oO][0-7_]*[0-7]/;
export const REG_BIN = /0[bB][01_]*[01]/;
export const REG_NUMBER = /\d[\d_]*(?:\.[\d_]+)?(?:[eE][+-]?[\d_]*\d)?/u;

/** 基础语言服务支持的最大插值字符串 `$` 数量 */
export const MAX_VERBATIM_LENGTH = 16;
