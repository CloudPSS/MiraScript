import type { VmContext, VmValue } from './index.ts';

const kVmScript = Symbol.for('mirascript.vm.script');

/** Mirascript 脚本 */
export type VmScriptLike = (global?: VmContext) => VmValue;

/** Mirascript 脚本 */
export type VmScript = VmScriptLike & {
    readonly [kVmScript]: true;
    /** 原始代码 */
    readonly source: string;
};

/** 检查是否为 Mirascript 脚本 */
export function isVmScript(value: unknown): value is VmScript {
    return typeof value === 'function' && (value as VmScript)[kVmScript] === true;
}
