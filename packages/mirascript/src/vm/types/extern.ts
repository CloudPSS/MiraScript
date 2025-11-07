import { VmError } from '../error.js';
import { VmWrapper } from './wrapper.js';
import type { TypeName, VmAny, VmConst, VmPrimitive, VmValue } from './index.js';
import { getPrototypeOf, hasOwn, apply } from '../../helpers/utils.js';
import { unwrapFromVmValue, wrapToVmValue } from './boundary.js';

const ObjectPrototype = Object.prototype;
// eslint-disable-next-line @typescript-eslint/unbound-method
const ObjectToString = ObjectPrototype.toString;
// eslint-disable-next-line @typescript-eslint/unbound-method
const FunctionToString = Function.prototype.toString;
/** 包装 Mirascript `extern` 类型的对象 */
export class VmExtern<const T extends object = object> extends VmWrapper<T> {
    constructor(
        value: T,
        readonly thisArg: T extends (...args: readonly never[]) => unknown ? VmExtern | null : null = null,
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

    /** 决定是否对属性进行包装 */
    protected assumeVmValue(value: object, key: keyof T | undefined): value is Exclude<VmConst, VmPrimitive> {
        return false;
    }

    /** @inheritdoc */
    override has(key: string): boolean {
        return this.access(key, true);
    }
    /** @inheritdoc */
    override get(key: string): VmAny {
        if (!this.has(key)) return undefined;
        const prop = (this.value as Record<string, unknown>)[key];
        return wrapToVmValue(prop, this, (v) => this.assumeVmValue(v, key as keyof T));
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
            throw VmError.from(`Not a callable extern`, null, null);
        }
        const caller = this.thisArg?.value ?? null;
        const unwrappedArgs = args.map(unwrapFromVmValue);
        let ret: unknown;
        try {
            ret = apply(value, caller, unwrappedArgs);
        } catch (ex) {
            throw VmError.from(`Callable extern`, ex, null);
        }
        return wrapToVmValue(ret, null, (obj) => this.assumeVmValue(obj, undefined));
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
        if (!isVmExtern(other)) return false;
        return this.value === other.value && this.thisArg === other.thisArg;
    }
    /** @inheritdoc */
    override toString(): string {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        const { toString } = this.value;
        if (typeof toString != 'function' || toString === ObjectToString || toString === FunctionToString) {
            return super.toString();
        }
        try {
            return String(this.value);
        } catch {
            return super.toString();
        }
    }
    /** @inheritdoc */
    override get type(): TypeName {
        return 'extern';
    }
    /** @inheritdoc */
    override get describe(): string {
        const tag = ObjectToString.call(this.value).slice(8, -1);
        if (tag === 'Object') {
            const proto = getPrototypeOf(this.value);
            if (proto === ObjectPrototype) {
                return 'Object';
            }
            if (proto == null) {
                return 'Object: null prototype';
            }
            if (typeof proto.constructor === 'function' && proto.constructor.name) {
                return proto.constructor.name;
            }
        } else if (tag === 'Function' && 'prototype' in this.value && typeof this.value.prototype == 'object') {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
            const { name } = this.value as unknown as Function;
            if (name) {
                return `Class ${name}`;
            } else {
                return 'Class';
            }
        }
        return tag;
    }
}

const kVmExtern = Symbol.for('mirascript.vm.extern');
Object.defineProperty(VmExtern.prototype, kVmExtern, { value: true });
/** 检查值是否为 Mirascript 外部值 */
export function isVmExtern<T extends object>(value: unknown): value is VmExtern<T> {
    return value != null && typeof value == 'object' && kVmExtern in value;
}
