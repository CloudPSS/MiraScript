import { VmFunction, type VmAny, type VmImmutable, type VmValue, wrapToVmValue, isVmAny } from '.';
const { getPrototypeOf, create, entries } = Object;

/** MiraScript 全局环境的基础，仅包含标准库 */
export type VmSharedGlobal = Record<string, VmImmutable>;
/** MiraScript 全局环境 */
export type VmGlobal = Record<string, VmValue | undefined>;

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
export function createVmGlobal(
    vmValues?: Record<string, VmValue | undefined>,
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
