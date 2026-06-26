import type * as wasm from './constants.g.js';
import { KEYWORDS } from './constants.g.js';

export {
    DiagnosticCode,
    OpCode,
    DIAGNOSTIC_MESSAGES,
    KEYWORDS,
    CONTROL_KEYWORDS,
    NUMERIC_KEYWORDS,
    CONSTANT_KEYWORDS,
    RESERVED_KEYWORDS,
} from './constants.g.js';
export * from './regex.js';

/** 配置选项 */
export type Config = Partial<
    Omit<wasm.Config, 'free' | 'input_mode' | 'diagnostic_position_encoding' | typeof Symbol.dispose>
> & {
    input_mode?: InputMode;
    diagnostic_position_encoding?: DiagnosticPositionEncoding;
};

/** 编译输入，支持字符串和 UTF-8 字节数组 */
export type ScriptInput = string | Uint8Array;
/** Encoding for counting positions in diagnostics. */
export type DiagnosticPositionEncoding = keyof typeof wasm.DiagnosticPositionEncoding;
/** 输入模式 */
export type InputMode = keyof typeof wasm.InputMode;

const keywordsSet = new Set<string>(KEYWORDS);
/** 检查是否为 MiraScript 关键字 */
export function isKeyword(word: string): word is (typeof KEYWORDS)[number] {
    if (typeof word !== 'string') return false;
    return keywordsSet.has(word);
}
