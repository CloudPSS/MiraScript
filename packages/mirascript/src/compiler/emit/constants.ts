export const SCRIPT_PREFIX = `((global) => { global ??= $GlobalFallback(); try { $CpEnter();`;
export const SCRIPT_PREFIX_NO_GLOBAL = `(() => { try { $CpEnter();`;
/** 脚本前缀类型 */
export type ScriptPrefix = typeof SCRIPT_PREFIX | typeof SCRIPT_PREFIX_NO_GLOBAL;
