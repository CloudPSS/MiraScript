import type { VmGlobal, VmValue } from './index.js';

const kVmScript = Symbol.for('mirascript.vm.script');

/** Mirascript 脚本 */
export type VmScript = {
    (global?: VmGlobal | null): VmValue;
    readonly [kVmScript]: true;
    /** 原始代码 */
    readonly source: string;
};

/** 检查是否为 Mirascript 脚本 */
export function isVmScript(value: unknown): value is VmScript {
    return typeof value === 'function' && (value as VmScript)[kVmScript] === true;
}
