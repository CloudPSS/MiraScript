import {
    VmFunction,
    type VmAny,
    type VmImmutable,
    type VmValue,
    wrapToVmValue,
    isVmAny,
    type VmFunctionLike,
} from './index.js';
import type * as global from '../lib/global.js';
const { getPrototypeOf, create, entries } = Object;
/** 全局导入的标准库 */
type GlobalKeys = keyof typeof global;
/** 全局导入的标准库名字 */
type StripUnderscore<T extends string> = T extends `_${infer R}_` ? R : T;
/** 全局导入的标准库值 */
type ToGlobalValue<T extends GlobalKeys> = (typeof global)[T] extends VmFunctionLike
    ? VmFunction<(typeof global)[T]>
    : (typeof global)[T];
/** 全局导入的标准库 */
type VmGlobalBase = {
    [key in GlobalKeys as StripUnderscore<key>]: ToGlobalValue<key>;
};
/** MiraScript 全局环境的基础，仅包含标准库 */
export type VmSharedGlobal = VmGlobalBase & Record<string, VmImmutable>;
/** MiraScript 全局环境 */
export type VmGlobal = VmGlobalBase & Record<string, VmValue | undefined>;

export const VmSharedGlobal = create(null) as VmSharedGlobal;

/** 定义在所有 MiraScript 全局环境中共享的全局函数 */
export function defineVmGlobalFunction(name: string, fn: (...args: VmAny[]) => VmAny, override = false): void {
    if (!override && name in VmSharedGlobal) throw new Error(`Global variable '${name}' is already defined.`);
    VmSharedGlobal[name] = VmFunction(fn, {
        isLib: true,
        fullName: `global.${name}`,
    });
}
/** 定义在所有 MiraScript 全局环境中共享的全局变量 */
export function defineVmGlobalValue(name: string, value: VmImmutable, override = false): void {
    if (!override && name in VmSharedGlobal) throw new Error(`Global variable '${name}' is already defined.`);
    VmSharedGlobal[name] = value ?? null;
}

/** 创建用于执行脚本的全局环境 */
export function createVmGlobal<const T extends Record<string, VmValue | undefined>>(
    vmValues?: T,
    externValues?: Record<string, unknown>,
): VmGlobal {
    const env = create(VmSharedGlobal) as VmGlobal;
    if (vmValues) {
        for (const [key, value] of entries(vmValues)) {
            if (!isVmAny(value, false)) continue;
            env[key] = value;
        }
    }
    if (externValues) {
        for (const [key, value] of entries(externValues)) {
            env[key] = value === undefined ? undefined : wrapToVmValue(value, null);
        }
    }
    return env;
}

/** 检查是否为全局环境 */
export function isVmGlobal(global: unknown): global is VmGlobal {
    if (global == null || typeof global != 'object') return false;
    return getPrototypeOf(global) === VmSharedGlobal;
}
