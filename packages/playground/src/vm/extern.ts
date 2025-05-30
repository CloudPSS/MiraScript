import { VmError } from './error';
import type { TypeName, VmAny, VmValue } from './types';
import { VmWrapper } from './wrapper';

const { apply } = Reflect;

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
        if (Object.hasOwn(this.value, key)) return true;
        if (!read) return true;
        const prop = (this.value as Record<string, unknown>)[key];
        if (key in Function.prototype && prop === Function.prototype[key as keyof (() => void)]) return false;
        if (key in Array.prototype && prop === Array.prototype[key as keyof unknown[]]) return false;
        if (key in Object.prototype && prop === Object.prototype[key as keyof object]) return false;
        return true;
    }

    /** 包装获取的值 */
    protected wrap(value: unknown): VmAny {
        if (value == null) return value;
        switch (typeof value) {
            case 'function':
                return new VmExtern(value, this);
            case 'object':
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

    /** @inheritdoc */
    override has(key: string): boolean {
        return this.access(key, true);
    }
    /** @inheritdoc */
    override get(key: string): VmAny {
        if (!this.has(key)) return undefined;
        const prop = (this.value as Record<string, unknown>)[key];
        return this.wrap(prop);
    }
    /** Set a property on the object */
    set(key: string, value: VmValue): boolean {
        if (!this.access(key, false)) return false;
        const prop = value instanceof VmWrapper ? value.value : value;
        (this.value as Record<string, unknown>)[key] = prop;
        return true;
    }
    /** Call extern value */
    call(args: readonly VmValue[]): VmAny {
        const { value } = this;
        if (typeof value != 'function') {
            throw new VmError(`Not a callable extern`);
        }
        const caller = this.caller?.value ?? null;
        const ret: unknown = apply(value, caller, args);
        return this.wrap(ret);
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
    override type(): TypeName {
        return 'extern';
    }
    /** @inheritdoc */
    override describe(): string {
        return Object.prototype.toString.call(this.value);
    }
}
