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

export const { REG_IDENTIFIER, REG_ORDINAL, REG_WHITESPACE, REG_BIN, REG_HEX, REG_OCT, REG_NUMBER } = constants;

/** 基础语言服务支持的最大插值字符串 `$` 数量 */
export const MAX_VERBATIM_LENGTH = 16;
