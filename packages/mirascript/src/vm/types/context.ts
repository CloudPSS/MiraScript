import {
    VmFunction,
    type VmAny,
    type VmImmutable,
    type VmValue,
    wrapToVmValue,
    isVmAny,
    type VmFunctionLike,
    isVmFunction,
} from './index.js';
import type * as global from '../lib/global/index.js';
const { entries } = Object;

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
const kVmContext = Symbol.for('mira:vm-context');
/** MiraScript 执行上下文的基础，仅包含标准库 */
export type VmSharedContext = VmContextBase & Record<string, VmImmutable>;
/** MiraScript 执行上下文 */
export interface VmContext {
    /** 内部标识符 */
    readonly [kVmContext]: true;
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

/** 定义在所有 MiraScript 执行上下文中共享的全局变量 */
export function defineVmContextValue(
    name: string,
    value: VmImmutable | ((...args: VmAny[]) => VmAny),
    override = false,
): void {
    if (!override && name in VmSharedContext) throw new Error(`Global variable '${name}' is already defined.`);
    let v: VmImmutable;
    if (typeof value == 'function' && !isVmFunction(value)) {
        v = VmFunction(value, {
            isLib: true,
            fullName: `global.${name}`,
        });
    } else {
        v = value;
    }
    VmSharedContext[name] = v ?? null;
}

/** 无后备的实现 */
export const DefaultVmContext: VmContext = Object.freeze({
    [kVmContext]: true as const,
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
    readonly [kVmContext] = true;
    /** @inheritdoc */
    keys(): Iterable<string> {
        this.cachedKeys ??= Object.keys(this.env);
        return [...this.cachedKeys, ...DefaultVmContext.keys()];
    }
    /** @inheritdoc */
    get(key: string): VmValue {
        return this.env[key] ?? null;
    }
    /** @inheritdoc */
    has(key: string): boolean {
        return key in this.env;
    }
    constructor(private readonly env: VmContextRecord & { __proto__: VmSharedContext }) {}
    private cachedKeys: readonly string[] | undefined;
}

/** 以工厂函数为后备的实现 */
class FactoryVmContext implements VmContext {
    readonly [kVmContext] = true;
    /** @inheritdoc */
    keys(): Iterable<string> {
        if (!this.enumerator) return DefaultVmContext.keys();
        return [...this.enumerator(), ...DefaultVmContext.keys()];
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
    constructor(
        private readonly getter: (key: string) => VmValue | undefined,
        private readonly enumerator?: () => Iterable<string>,
    ) {}
}

/** 创建用于执行脚本的执行上下文 */
export function createVmContext(
    ...args:
        | readonly [vmValues?: VmContextRecord, externValues?: Record<string, unknown>]
        | readonly [getter: (key: string) => VmValue | undefined, enumerator?: () => Iterable<string>]
): VmContext {
    if (args[0] == null && args[1] == null) {
        return { ...DefaultVmContext };
    }

    if (typeof args[0] == 'function') {
        return new FactoryVmContext(args[0], args[1] as (() => Iterable<string>) | undefined);
    }

    const [vmValues, externValues] = args;
    const env = { __proto__: VmSharedContext } as { __proto__: VmSharedContext } & VmContextRecord;
    if (vmValues) {
        for (const [key, value] of entries(vmValues)) {
            if (!isVmAny(value, false)) continue;
            env[key] = value ?? null;
        }
    }
    if (externValues) {
        for (const [key, value] of entries(externValues as Record<string, unknown>)) {
            env[key] = value == null ? null : wrapToVmValue(value, null);
        }
    }
    return new ValueVmContext(env);
}

/** 检查是否为执行上下文 */
export function isVmContext(context: unknown): context is VmContext {
    if (context == null || typeof context != 'object') return false;
    return (context as VmContext)[kVmContext] === true;
}
