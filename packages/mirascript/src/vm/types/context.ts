import {
    VmFunction,
    type VmAny,
    type VmImmutable,
    type VmValue,
    wrapToVmValue,
    isVmAny,
    type VmFunctionLike,
} from './index.js';
import type * as global from '../lib/global/index.js';
const { getPrototypeOf, entries } = Object;

/** 全局导入的标准库 */
type GlobalKeys = keyof typeof global;
/** 全局导入的标准库值 */
type ToGlobalValue<T extends GlobalKeys> = (typeof global)[T] extends VmFunctionLike
    ? VmFunction<(typeof global)[T]>
    : (typeof global)[T];
/** 全局导入的标准库 */
type VmContextBase = {
    [key in GlobalKeys]: ToGlobalValue<key>;
};
/** MiraScript 执行上下文的基础，仅包含标准库 */
export type VmSharedContext = VmContextBase & Record<string, VmImmutable>;
/** MiraScript 执行上下文 */
export interface VmContext {
    /** 枚举所有 key，仅在 LSP 中使用 */
    keys(): Iterable<string>;
    /** 获取指定 key 的值 `global[key]` */
    get(key: string): VmValue;
    /** 查找指定 key 是否存在 `key in global` */
    has(key: string): boolean;
}
/** MiraScript 执行上下文 */
export type VmContextRecord = Record<string, VmValue | undefined>;
export const VmSharedContext = { __proto__: null } as object as VmSharedContext;

/** 定义在所有 MiraScript 执行上下文中共享的全局函数 */
export function defineVmContextFunction(name: string, fn: (...args: VmAny[]) => VmAny, override = false): void {
    if (!override && name in VmSharedContext) throw new Error(`Global variable '${name}' is already defined.`);
    VmSharedContext[name] = VmFunction(fn, {
        isLib: true,
        fullName: `global.${name}`,
    });
}
/** 定义在所有 MiraScript 执行上下文中共享的全局变量 */
export function defineVmContextValue(name: string, value: VmImmutable, override = false): void {
    if (!override && name in VmSharedContext) throw new Error(`Global variable '${name}' is already defined.`);
    VmSharedContext[name] = value ?? null;
}

/** 无后备的实现 */
export const DefaultVmContext: VmContext = Object.freeze({
    /** @inheritdoc */
    keys(): Iterable<string> {
        return Object.keys(VmSharedContext);
    },
    /** @inheritdoc */
    get(key: string): VmValue {
        return VmSharedContext[key] ?? null;
    },
    /** @inheritdoc */
    has(key: string): boolean {
        return key in VmSharedContext;
    },
});

/** 以值为后备的实现 */
class ValueVmContext implements VmContext {
    constructor(private readonly env: VmContextRecord) {}
    private cachedKeys: readonly string[] | undefined;
    /** @inheritdoc */
    keys(): Iterable<string> {
        this.cachedKeys ??= Object.keys(this.env);
        return [...this.cachedKeys, ...DefaultVmContext.keys()];
    }
    /** @inheritdoc */
    get(key: string): VmValue {
        if (key in this.env) return this.env[key] ?? null;
        return DefaultVmContext.get(key);
    }
    /** @inheritdoc */
    has(key: string): boolean {
        return key in this.env || DefaultVmContext.has(key);
    }
}

/** 以工厂函数为后备的实现 */
class FactoryVmContext implements VmContext {
    constructor(
        private readonly getter: (key: string) => VmValue | undefined,
        private readonly enumerator?: () => Iterable<string>,
    ) {}
    /** @inheritdoc */
    keys(): Iterable<string> {
        return [...(this.enumerator ? this.enumerator() : []), ...DefaultVmContext.keys()];
    }
    /** @inheritdoc */
    get(key: string): VmValue {
        const value = this.getter(key);
        if (value !== undefined) return value;
        return DefaultVmContext.get(key);
    }
    /** @inheritdoc */
    has(key: string): boolean {
        return this.getter(key) !== undefined || DefaultVmContext.has(key);
    }
}

/** 创建用于执行脚本的执行上下文 */
export function createVmContext(
    ...args:
        | [vmValues?: VmContextRecord, externValues?: Record<string, unknown>]
        | [getter: (key: string) => VmValue | undefined, enumerator?: () => Iterable<string>]
): VmContext {
    if (typeof args[0] == 'function') {
        return new FactoryVmContext(args[0], args[1] as (() => Iterable<string>) | undefined);
    } else {
        const [vmValues, externValues] = args;
        if (vmValues == null && externValues == null) {
            return { ...DefaultVmContext };
        }
        const env = { __proto__: VmSharedContext } as object as VmContextRecord;
        if (vmValues) {
            for (const [key, value] of entries(vmValues)) {
                if (!isVmAny(value, false)) continue;
                env[key] = value;
            }
        }
        if (externValues) {
            for (const [key, value] of entries(externValues as Record<string, unknown>)) {
                env[key] = value === undefined ? undefined : wrapToVmValue(value, null);
            }
        }
        return new ValueVmContext(env);
    }
}

/** 检查是否为执行上下文 */
export function isVmContext(context: unknown): context is VmContext {
    if (context == null || typeof context != 'object') return false;
    return getPrototypeOf(context) === VmSharedContext || context === VmSharedContext;
}
