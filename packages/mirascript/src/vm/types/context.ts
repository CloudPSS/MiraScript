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
type VmContextBase = {
    [key in GlobalKeys as StripUnderscore<key>]: ToGlobalValue<key>;
};
/** MiraScript 执行上下文的基础，仅包含标准库 */
export type VmSharedContext = VmContextBase & Record<string, VmImmutable>;
/** MiraScript 执行上下文 */
export type VmContext = VmContextBase & Record<string, VmValue | undefined>;

export const VmSharedContext = create(null) as VmSharedContext;

/** 定义在所有 MiraScript 执行上下文中共享的全局函数 */
export function defineVmGlobalFunction(name: string, fn: (...args: VmAny[]) => VmAny, override = false): void {
    if (!override && name in VmSharedContext) throw new Error(`Global variable '${name}' is already defined.`);
    VmSharedContext[name] = VmFunction(fn, {
        isLib: true,
        fullName: `global.${name}`,
    });
}
/** 定义在所有 MiraScript 执行上下文中共享的全局变量 */
export function defineVmGlobalValue(name: string, value: VmImmutable, override = false): void {
    if (!override && name in VmSharedContext) throw new Error(`Global variable '${name}' is already defined.`);
    VmSharedContext[name] = value ?? null;
}

/** 创建用于执行脚本的执行上下文 */
export function createVmContext<const T extends Record<string, VmValue | undefined>>(
    vmValues?: T,
    externValues?: Record<string, unknown>,
): VmContext {
    const env = create(VmSharedContext) as VmContext;
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

/** 检查是否为执行上下文 */
export function isVmContext(context: unknown): context is VmContext {
    if (context == null || typeof context != 'object') return false;
    return getPrototypeOf(context) === VmSharedContext;
}
