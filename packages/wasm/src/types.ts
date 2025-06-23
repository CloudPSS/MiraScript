import type * as wasm from '../lib/wasm.js';
export { DiagnosticCode, OpCode } from '../lib/wasm.js';

/**
 * 配置选项
 */
export type Config = Partial<Omit<wasm.Config, 'free' | 'input_mode' | 'diagnostic_position_encoding'>> & {
    input_mode?: InputMode;
    diagnostic_position_encoding?: DiagnosticPositionEncoding;
};
/** Encoding for counting positions in diagnostics. */
export type DiagnosticPositionEncoding = keyof typeof wasm.DiagnosticPositionEncoding;
/** 输入模式 */
export type InputMode = keyof typeof wasm.InputMode;
/** 编译输入，支持字符串和 UTF-8 字节数组 */
export type ScriptInput = string | Uint8Array;
