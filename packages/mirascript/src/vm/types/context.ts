import { create, entries, keys } from '../../helpers/utils.js';
import { isVmAny } from '../../helpers/types.js';
import { VmError } from '../../helpers/error.js';
import { kVmContext } from '../../helpers/constants.js';
import type * as global from '../lib/global/index.js';
import type { VmAny, VmImmutable, VmValue, VmFunctionLike } from './index.js';
import { wrapToVmValue } from './boundary.js';
import { VmFunction } from './function.js';

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
    /** 内部标识符 */
    readonly [kVmContext]: true;
    /** 枚举所有 key，仅在 LSP 中使用 */
    keys(): Iterable<string>;
    /** 描述值，返回 MarkDown 文本，仅在 LSP 中使用 */
    describe?(key: string): string | undefined;
    /**
     * 获取指定 key 的值 `global[key]`
     * @throws {VmError} 如果值不存在则抛出异常
     */
    get(key: string): VmValue;
    /** 查找指定 key 是否存在 `key in global` */
    has(key: string): boolean;
}
/** MiraScript 执行上下文 */
export type VmContextRecord = Record<string, VmValue>;
/** MiraScript 执行上下文 */
export type VmContextRecordLoose = Record<string, VmValue | undefined>;
export const VmSharedContext = create(null) as VmSharedContext;

let VmSharedContextKeys: readonly string[] | null = null;

/** 全局变量未找到 */
function globalVarNotFound(name: string): never {
    throw new VmError(`Global variable '${name}' is not defined.`, null);
}

/** 定义在所有 MiraScript 执行上下文中共享的全局变量 */
export function defineVmContextValue(
    name: string,
    value: VmImmutable | ((...args: VmAny[]) => VmAny),
    override = false,
): void {
    if (!override && name in VmSharedContext) throw new Error(`Global variable '${name}' is already defined.`);
    let v: VmImmutable;
    if (typeof value == 'function') {
        v = VmFunction(value, {
            isLib: true,
            fullName: `global.${name}`,
        });
    } else {
        v = value;
    }
    VmSharedContext[name] = v ?? null;
    VmSharedContextKeys = null;
}

/** 无后备的实现 */
export const DefaultVmContext: VmContext = Object.freeze({
    [kVmContext]: true as const,
    /** @inheritdoc */
    keys(): Iterable<string> {
        VmSharedContextKeys ??= Object.freeze(keys(VmSharedContext));
        return VmSharedContextKeys;
    },
    /** @inheritdoc */
    get(key: string): VmValue {
        const val = VmSharedContext[key];
        if (val === undefined) globalVarNotFound(key);
        return val;
    },
    /** @inheritdoc */
    has(key: string): boolean {
        return key in VmSharedContext;
    },
});

/** 以值为后备的实现 */
class ValueVmContext implements VmContext {
    readonly [kVmContext] = true;
    private cachedKeys: readonly string[] | null = null;
    /** @inheritdoc */
    keys(): Iterable<string> {
        this.cachedKeys ??= keys(this.env);
        return [...this.cachedKeys, ...DefaultVmContext.keys()];
    }
    /** @inheritdoc */
    get(key: string): VmValue {
        const val = this.env[key];
        if (val === undefined) globalVarNotFound(key);
        return val;
    }
    /** @inheritdoc */
    has(key: string): boolean {
        return key in this.env;
    }
    constructor(
        private readonly env: VmContextRecord,
        /** @inheritdoc */
        readonly describe?: (key: string) => string | undefined,
    ) {}
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
        /** @inheritdoc */
        readonly describe?: (key: string) => string | undefined,
    ) {}
}

/** 以值为后备的实现 */
type CreateVmContextWithValues = readonly [
    vmValues?: VmContextRecordLoose | null | undefined,
    externValues?: Record<string, unknown> | null | undefined,
    describer?: ((key: string) => string | undefined) | null | undefined,
];
/** 以工厂函数为后备的实现 */
type CreateVmContextWithFactory = readonly [
    getter: (key: string) => VmValue | undefined,
    enumerator?: (() => Iterable<string>) | null | undefined,
    describer?: ((key: string) => string | undefined) | null | undefined,
];

/** 创建用于执行脚本的执行上下文 */
export function createVmContext(...args: CreateVmContextWithValues | CreateVmContextWithFactory): VmContext {
    if (args[0] == null && args[1] == null) {
        return { ...DefaultVmContext };
    }

    if (typeof args[0] == 'function') {
        const [getter, enumerator, describer] = args as CreateVmContextWithFactory;
        return new FactoryVmContext(getter, enumerator ?? undefined, describer ?? undefined);
    }

    const [vmValues, externValues, describer] = args as CreateVmContextWithValues;
    const env = create(VmSharedContext) as VmContextRecord;
    if (vmValues) {
        for (const [key, value] of entries(vmValues)) {
            if (!isVmAny(value, false)) continue;
            env[key] = value ?? null;
        }
    }
    if (externValues) {
        for (const [key, value] of entries(externValues)) {
            env[key] = value == null ? null : wrapToVmValue(value, null);
        }
    }
    return new ValueVmContext(env, describer ?? undefined);
}
