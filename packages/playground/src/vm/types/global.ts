import {
    VmFunction,
    type VmExtern,
    type VmModule,
    type VmAny,
    type VmImmutable,
    type VmPrimitive,
    type VmValue,
    wrapToVmValue,
} from '.';
const { getPrototypeOf, create, keys } = Object;

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
    override?: Record<string, VmFunction | VmExtern | VmPrimitive | VmModule | object>,
): VmGlobal {
    const env = create(VmSharedGlobal) as VmGlobal;
    if (override) {
        for (const key of keys(override)) {
            const value = override[key];
            env[key] = wrapToVmValue(value, null);
        }
    }
    return env;
}

/** 检查是否为全局环境 */
export function isVmGlobal(global: unknown): global is VmGlobal {
    return getPrototypeOf(global) === VmSharedGlobal;
}
