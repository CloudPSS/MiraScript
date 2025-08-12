import { VmError } from '../error.js';
import { isVmFunction, type TypeName, type VmAny, type VmModule, type VmValue } from './index.js';
import { VmWrapper } from './wrapper.js';

const { apply } = Reflect;
const { hasOwn } = Object;

/** 包装为 Mirascript 类型 */
export function wrapToVmValue(value: unknown, caller: VmExtern | null): VmValue {
    if (value == null) return null;
    switch (typeof value) {
        case 'function':
            if (isVmFunction(value)) return value;
            return new VmExtern(value, caller);
        case 'object':
            if (value instanceof VmWrapper) return value as VmModule | VmExtern;
            return new VmExtern(value, null);
        case 'string':
        case 'number':
        case 'boolean':
            return value;
        case 'bigint':
            return Number(value);
        case 'symbol':
        case 'undefined':
        default:
            return null;
    }
}

/** 取消 Mirascript 类型包装  */
export function unwrapFromVmValue(value: VmAny): unknown {
    if (value == null || typeof value != 'object') return value;
    if (!(value instanceof VmExtern)) return value;
    if (value.caller == null || typeof value.value != 'function') {
        return value.value;
    }
    const caller = value.caller.value;
    const func = value.value as (...args: unknown[]) => unknown;
    return new Proxy(func, {
        apply(target, thisArg, args): unknown {
            return apply(target, caller, args);
        },
    });
}

/** 包装 Mirascript `extern` 类型的对象 */
export class VmExtern<const T extends object = object> extends VmWrapper<T> {
    constructor(
        value: T,
        readonly caller: VmExtern | null = null,
    ) {
        super(value);
    }

    /** Check if the object has a property */
    protected access(key: string, read: boolean): boolean {
        // __proto__ and other private properties are not accessible
        if (key.startsWith('_')) return false;
        // Function-specific properties are not accessible
        if (typeof this.value == 'function' && (key === 'prototype' || key === 'arguments' || key === 'caller'))
            return false;
        if (hasOwn(this.value, key)) return true;
        if (!read) return true;
        if (!(key in this.value)) return false;
        if (key === 'constructor') return false; // constructor is not accessible
        const prop = (this.value as Record<string, unknown>)[key];
        if (key in Function.prototype && prop === Function.prototype[key as keyof (() => void)]) return false;
        if (key in Array.prototype && prop === Array.prototype[key as keyof unknown[]]) return false;
        if (key in Object.prototype && prop === Object.prototype[key as keyof object]) return false;
        return true;
    }

    /** @inheritdoc */
    override has(key: string): boolean {
        return this.access(key, true);
    }
    /** @inheritdoc */
    override get(key: string): VmAny {
        if (!this.has(key)) return undefined;
        const prop = (this.value as Record<string, unknown>)[key];
        return wrapToVmValue(prop, this);
    }
    /** Set a property on the object */
    set(key: string, value: VmValue): boolean {
        if (!this.access(key, false)) return false;
        const prop = unwrapFromVmValue(value);
        (this.value as Record<string, unknown>)[key] = prop;
        return true;
    }
    /** Call extern value */
    call(args: readonly VmValue[]): VmAny {
        const { value } = this;
        if (typeof value != 'function') {
            throw new VmError(`Not a callable extern`, null);
        }
        const caller = this.caller?.value ?? null;
        const unwrappedArgs = args.map(unwrapFromVmValue);
        const ret: unknown = apply(value, caller, unwrappedArgs);
        return wrapToVmValue(ret, this);
    }
    /** Can extern value be called */
    get callable(): boolean {
        return typeof this.value === 'function';
    }
    /** @inheritdoc */
    override keys(): string[] {
        const keys: string[] = [];
        for (const key in this.value) {
            if (this.has(key)) keys.push(key);
        }
        return keys;
    }
    /** @inheritdoc */
    override same(other: VmAny): boolean {
        if (!(other instanceof VmExtern)) return false;
        return this.value === other.value && this.caller === other.caller;
    }
    /** @inheritdoc */
    override get type(): TypeName {
        return 'extern';
    }
    /** @inheritdoc */
    override get describe(): string {
        return Object.prototype.toString.call(this.value).slice(8, -1);
    }
}
