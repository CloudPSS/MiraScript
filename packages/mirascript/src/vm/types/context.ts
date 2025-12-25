import { create, defineProperty, hasOwn, keys } from '../../helpers/utils.js';
import { VmError } from '../../helpers/error.js';
import { kVmContext } from '../../helpers/constants.js';
import type { VmAny, VmImmutable, VmValue } from './index.js';
import { wrapToVmValue } from './boundary.js';
import { VmFunction } from './function.js';
const { freeze } = Object;

/** MiraScript 执行上下文 */
export interface VmContext {
    /** 内部标识符 */
    readonly [kVmContext]: true;
    /** 枚举所有 key，仅在 LSP 中使用 */
    keys(): readonly string[];
    /** 描述值，返回 MarkDown 文本，仅在 LSP 中使用 */
    describe(key: string): string | undefined;
    /**
     * 获取指定 key 的值 `global[key]`
     * @throws {VmError} 如果值不存在则抛出异常
     */
    get(key: string): VmValue;
    /** 查找指定 key 是否存在 `key in global` */
    has(key: string): boolean;
}
/** MiraScript 执行上下文 */
export type VmContextRecord = Record<string, VmAny>;
export const VM_SHARED_CONTEXT: Record<string, VmImmutable> = create(null);
export const VM_SHARED_CONTEXT_DESCRIPTIONS: Record<string, string | undefined> = create(null);
/** 缓存 {@link VM_SHARED_CONTEXT} 的 keys */
let VM_SHARED_CONTEXT_KEYS: readonly string[] | null = null;

/** 全局变量未找到 */
function globalVarNotFound(name: string): never {
    throw new VmError(`Global variable '${name}' is not defined.`, null);
}

/** 定义在所有 MiraScript 执行上下文中共享的全局变量 */
export function defineVmContextValue(
    name: string,
    value: VmImmutable | ((...args: VmAny[]) => VmAny),
    override = false,
    description: string | null | undefined = undefined,
): void {
    if (!override && name in VM_SHARED_CONTEXT) throw new Error(`Global variable '${name}' is already defined.`);
    let v: VmImmutable;
    if (typeof value == 'function') {
        v = VmFunction(value, {
            isLib: true,
            fullName: `global.${name}`,
        });
    } else {
        v = value;
    }
    VM_SHARED_CONTEXT[name] = v ?? null;
    if (description) VM_SHARED_CONTEXT_DESCRIPTIONS[name] = description;
    VM_SHARED_CONTEXT_KEYS = null;
}

/** 无后备的实现 */
export const DefaultVmContext: Readonly<VmContext> = freeze({
    __proto__: null,
    [kVmContext]: true as const,
    /** @inheritdoc */
    keys(): readonly string[] {
        VM_SHARED_CONTEXT_KEYS ??= freeze(keys(VM_SHARED_CONTEXT));
        return VM_SHARED_CONTEXT_KEYS;
    },
    /** @inheritdoc */
    get(key: string): VmValue {
        const val = VM_SHARED_CONTEXT[key];
        if (val !== undefined) return val;
        return globalVarNotFound(key);
    },
    /** @inheritdoc */
    has(key: string): boolean {
        return key in VM_SHARED_CONTEXT;
    },
    /** @inheritdoc */
    describe(key: string): string | undefined {
        return VM_SHARED_CONTEXT_DESCRIPTIONS[key];
    },
});

/** 以值为后备的实现 */
class ValueVmContext implements VmContext {
    declare readonly [kVmContext]: true;
    /** @inheritdoc */
    keys(): readonly string[] {
        return [...keys(this.env), ...DefaultVmContext.keys()];
    }
    /** @inheritdoc */
    get(key: string): VmValue {
        if (hasOwn(this.env, key)) return this.env[key] ?? null;
        {
            const val = VM_SHARED_CONTEXT[key];
            if (val !== undefined) return val;
            return globalVarNotFound(key);
        }
    }
    /** @inheritdoc */
    has(key: string): boolean {
        return hasOwn(this.env, key) || key in VM_SHARED_CONTEXT;
    }
    /** @inheritdoc */
    describe(key: string): string | undefined {
        if (hasOwn(this.env, key)) return this.describer?.(key);
        return VM_SHARED_CONTEXT_DESCRIPTIONS[key];
    }
    constructor(
        private readonly env: VmContextRecord,
        private readonly describer: ((key: string) => string | undefined) | null,
    ) {}
}
defineProperty(ValueVmContext.prototype, kVmContext, { value: true });
freeze(ValueVmContext.prototype);

/** 以值为后备的实现 */
class Value2VmContext implements VmContext {
    declare readonly [kVmContext]: true;
    /** @inheritdoc */
    keys(): readonly string[] {
        return [...(this.env == null ? [] : keys(this.env)), ...keys(this.extern), ...DefaultVmContext.keys()];
    }
    /** @inheritdoc */
    get(key: string): VmValue {
        if (this.env != null && hasOwn(this.env, key)) return this.env[key] ?? null;
        if (hasOwn(this.extern, key)) {
            const val = this.extern[key];
            if (val == null) return null;
            if (typeof val != 'object' && typeof val != 'function') {
                return wrapToVmValue(val, null, null);
            }
            let cached = this.externCache.get(val);
            if (cached == null) {
                cached = wrapToVmValue(val, null, null);
                this.externCache.set(val, cached);
            }
            return cached;
        }

        const val = VM_SHARED_CONTEXT[key];
        if (val !== undefined) return val;
        return globalVarNotFound(key);
    }
    /** @inheritdoc */
    has(key: string): boolean {
        return (this.env != null && hasOwn(this.env, key)) || hasOwn(this.extern, key) || key in VM_SHARED_CONTEXT;
    }
    /** @inheritdoc */
    describe(key: string): string | undefined {
        if ((this.env != null && hasOwn(this.env, key)) || hasOwn(this.extern, key)) return this.describer?.(key);
        return VM_SHARED_CONTEXT_DESCRIPTIONS[key];
    }
    private readonly externCache = new WeakMap<object, VmValue>();
    constructor(
        private readonly env: VmContextRecord | null,
        private readonly extern: Record<string, unknown>,
        private readonly describer: ((key: string) => string | undefined) | null,
    ) {}
}
defineProperty(Value2VmContext.prototype, kVmContext, { value: true });
freeze(Value2VmContext.prototype);

/** 以工厂函数为后备的实现 */
class FactoryVmContext implements VmContext {
    declare readonly [kVmContext]: true;
    /** @inheritdoc */
    keys(): readonly string[] {
        if (!this.enumerator) return DefaultVmContext.keys();
        return [...this.enumerator(), ...DefaultVmContext.keys()];
    }
    /** @inheritdoc */
    get(key: string): VmValue {
        const value = this.getter(key);
        if (value !== undefined) return value;

        const val = VM_SHARED_CONTEXT[key];
        if (val !== undefined) return val;
        return globalVarNotFound(key);
    }
    /** @inheritdoc */
    has(key: string): boolean {
        return this.getter(key) !== undefined || key in VM_SHARED_CONTEXT;
    }
    /** @inheritdoc */
    describe(key: string): string | undefined {
        if (this.getter(key) !== undefined) return this.describer?.(key);
        return VM_SHARED_CONTEXT_DESCRIPTIONS[key];
    }
    constructor(
        private readonly getter: (key: string) => VmValue | undefined,
        private readonly enumerator: (() => Iterable<string>) | null,
        private readonly describer: ((key: string) => string | undefined) | null,
    ) {}
}
defineProperty(FactoryVmContext.prototype, kVmContext, { value: true });
freeze(FactoryVmContext.prototype);

/** 获取用于执行脚本的默认执行上下文 */
export function createVmContext(): Readonly<VmContext>;
/** 创建用于执行脚本的执行上下文 */
export function createVmContext(
    getter: (key: string) => VmValue | undefined,
    enumerator?: (() => Iterable<string>) | null,
    describer?: ((key: string) => string | undefined) | null,
): VmContext;
/** 创建用于执行脚本的执行上下文 */
export function createVmContext(
    vmValues?: VmContextRecord | null,
    externValues?: Record<string, unknown> | null,
    describer?: ((key: string) => string | undefined) | null,
): VmContext;

/** 创建用于执行脚本的执行上下文 */
export function createVmContext(
    arg0: VmContextRecord | ((key: string) => VmValue | undefined) | null = null,
    arg1: Record<string, unknown> | (() => Iterable<string>) | null = null,
    describer: ((key: string) => string | undefined) | null = null,
): VmContext {
    if (typeof arg0 == 'function') {
        const getter = arg0;
        const enumerator = arg1 as (() => Iterable<string>) | null;
        return new FactoryVmContext(getter, enumerator, describer);
    }

    const vmValues = arg0;
    const externValues = arg1 as Record<string, unknown> | null;
    if (externValues == null) {
        if (vmValues == null) {
            return DefaultVmContext;
        }
        return new ValueVmContext(vmValues, describer);
    }
    return new Value2VmContext(vmValues, externValues, describer);
}
