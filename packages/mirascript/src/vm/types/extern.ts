import { VmError } from '../../helpers/error.js';
import { getPrototypeOf, getOwnPropertyNames, hasOwn, apply, isArray } from '../../helpers/utils.js';
import { innerToString } from '../../helpers/convert/to-string.js';
import { isVmExtern } from '../../helpers/types.js';
import { kVmExtern } from '../../helpers/constants.js';
import type { VmTypeName, VmAny, VmConst, VmPrimitive, VmValue } from './index.js';
import { VmWrapper } from './wrapper.js';
import { unwrapFromVmValue, wrapToVmValue } from './boundary.js';

const ObjectPrototype = Object.prototype;
// eslint-disable-next-line @typescript-eslint/unbound-method
const ObjectToString = ObjectPrototype.toString;
// eslint-disable-next-line @typescript-eslint/unbound-method
const FunctionToString = Function.prototype.toString;
const ArrayToString = Array.prototype.toString;
const ArrayMap = Array.prototype.map;
// eslint-disable-next-line @typescript-eslint/unbound-method
const TypedArrayToString = Uint8Array.prototype.toString;

/** 获取类的名称，如果无法确定则返回 null */
// eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
function classNameOf(kls: Function): string | null {
    const { displayName } = kls as typeof kls & { displayName?: string };
    if (typeof displayName === 'string' && displayName.trim()) {
        return displayName.trim();
    }
    const { name } = kls;
    if (typeof name == 'string' && name.length > 2) {
        // Looks like a non-minified name
        return name;
    }
    return null;
}

/** 包装 Mirascript `extern` 类型的对象 */
export class VmExtern<const T extends object = object> extends VmWrapper<T> {
    constructor(
        /** 包装值 */
        value: T,
        /** 当 {@link value} 是函数时，绑定的 this 参数 */
        readonly thisArg: ThisParameterType<T> | null = null,
    ) {
        super(value);
    }

    /**
     * Check if the object has a property
     * This method will be used in {@link get}, {@link set}, {@link has}, and {@link keys} methods
     */
    protected access(key: string, read: boolean): boolean {
        // __proto__ and other “private” properties are not accessible
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
    assumeVmValue(value: object, key: keyof T | undefined): value is Exclude<VmConst, VmPrimitive> {
        return false;
    }

    /** @inheritdoc */
    override has(key: string): boolean {
        return this.access(key, true);
    }
    /** @inheritdoc */
    override get(key: string): VmAny {
        if (!(key in this.value)) return undefined;
        if (!this.access(key, true)) return undefined;
        const prop = (this.value as Record<string, unknown>)[key];
        return wrapToVmValue(prop, this.value, (v) => this.assumeVmValue(v, key as keyof T));
    }
    /** Set a property on the object */
    set(key: string, value: VmValue): boolean {
        if (!this.access(key, false)) return false;
        const prop = unwrapFromVmValue(value, true);
        (this.value as Record<string, unknown>)[key] = prop;
        return true;
    }
    /** Call extern value */
    call(args: readonly VmValue[]): VmAny {
        const { value } = this;
        if (typeof value != 'function') {
            throw VmError.from(`Not a callable extern`, null, null);
        }
        const caller = this.thisArg;
        const unwrappedArgs = args.map((arg) => unwrapFromVmValue(arg, true));
        let ret: unknown;
        try {
            ret = apply(value, caller, unwrappedArgs);
        } catch (ex) {
            throw VmError.from(`Callable extern`, ex, null);
        }
        return wrapToVmValue(ret, null, (obj) => this.assumeVmValue(obj, undefined));
    }
    /** @inheritdoc */
    override keys(includeNonEnumerable = false): string[] {
        if (!includeNonEnumerable) {
            const keys: string[] = [];
            for (const key in this.value) {
                if (this.access(key, true)) keys.push(key);
            }
            return keys;
        } else {
            const keys = new Set<string>();
            let e: unknown = this.value;
            while (e != null && (typeof e == 'object' || typeof e == 'function')) {
                for (const key of getOwnPropertyNames(e)) {
                    keys.add(key);
                }
                e = getPrototypeOf(e);
            }
            return Array.from(keys).filter((key) => this.access(key, true));
        }
    }
    /** @inheritdoc */
    override same(other: VmAny): boolean {
        if (!isVmExtern(other)) return false;
        return this.value === other.value && this.thisArg === other.thisArg;
    }
    /**
     * Should this extern be treated as array-like?
     *
     * By default, this method returns true if the wrapped value is an Array or a TypedArray.
     */
    isArrayLike(): this is VmExtern<ArrayLike<unknown>> {
        return isArray(this.value) || (ArrayBuffer.isView(this.value) && 'length' in this.value);
    }
    /** @inheritdoc */
    override toString(useBraces: boolean): string {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        const { toString } = this.value;
        if (typeof toString != 'function' || toString === ObjectToString || toString === FunctionToString) {
            // When the toString method is not overridden or invalid, provide a better default representation
            return super.toString(useBraces);
        }
        if ((toString === ArrayToString || toString === TypedArrayToString) && this.isArrayLike()) {
            // Handle array-like externs specially when using default toString
            const mapped = ArrayMap.call(this.value, (item: unknown) => {
                if (item === undefined) return '';
                return innerToString(wrapToVmValue(item ?? null, null, null), true);
            });
            const str = mapped.join(', ');
            if (useBraces) return `[${str}]`;
            return str;
        }
        // Use the wrapped object's toString method
        return String(this.value);
    }
    /** @inheritdoc */
    override get type(): VmTypeName {
        return 'extern';
    }
    /** @inheritdoc */
    override get tag(): string {
        const tag = ObjectToString.call(this.value).slice(8, -1);
        if (this.isArrayLike()) {
            return `${tag}(${this.value.length})`;
        } else if (tag === 'Object') {
            const proto = getPrototypeOf(this.value);
            if (proto === ObjectPrototype) {
                return 'Object';
            }
            if (proto == null) {
                return 'Object: null prototype';
            }
            if (typeof proto.constructor === 'function') {
                return classNameOf(proto.constructor) ?? 'Object';
            }
        } else if (tag === 'Function' && 'prototype' in this.value && typeof this.value.prototype == 'object') {
            const className = classNameOf(this.value as unknown as new () => unknown);
            if (!className) return `class`;
            return `class ${className}`;
        } else if (tag === 'Function') {
            return `function`;
        } else if (tag === 'AsyncFunction') {
            return `async function`;
        } else if (tag === 'GeneratorFunction') {
            return `function*`;
        } else if (tag === 'AsyncGeneratorFunction') {
            return `async function*`;
        }
        return tag;
    }
}

Object.defineProperty(VmExtern.prototype, kVmExtern, { value: true });
