import type { VmContext, VmValue } from './index.js';

declare const kVmScript: unique symbol;

/** Mirascript 脚本 */
export type VmScriptLike = (global?: VmContext) => VmValue;

/** Mirascript 脚本 */
export type VmScript = VmScriptLike & {
    readonly [kVmScript]: true;
    /** 原始代码 */
    readonly source: string;
};
