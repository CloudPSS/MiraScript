import { VmError } from '../../../helpers/error.js';
import { apply } from '../../../helpers/utils.js';
import { innerToString } from '../../../helpers/convert/string.js';
import { isVmExtern } from '../../../helpers/types/index.js';
import { kVmExtern } from '../../../helpers/constants.js';
import type { VmTypeName, VmAny, VmConst, VmPrimitive, VmValue } from '../index.js';
import { VmWrapper } from '../wrapper.js';
import { unwrapFromVmValue, wrapToVmValue } from '../boundary.js';
import { getTag, hasCustomToString } from './repr.js';
import { canAccessProperty, getKeys, isArrayLike } from './utils.js';

const ArrayMap = Array.prototype.map;

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
        return canAccessProperty(this.value, key, read);
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
        const keys = getKeys(this.value, includeNonEnumerable);
        return keys.filter((key) => this.access(key, true));
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
        return isArrayLike(this.value);
    }
    /** @inheritdoc */
    override toString(useBraces: boolean): string {
        if (hasCustomToString(this.value)) {
            // Use the wrapped object's toString method
            return String(this.value);
        }
        if (this.isArrayLike()) {
            // Handle array-like externs specially when using default toString
            const mapped = ArrayMap.call(this.value, (item: unknown) => {
                if (item === undefined) return '';
                return innerToString(wrapToVmValue(item ?? null, null, null), true);
            });
            const str = mapped.join(', ');
            if (useBraces) return `[${str}]`;
            return str;
        }
        // When the toString method is not overridden or invalid, provide a better default representation
        return super.toString(useBraces);
    }
    /** @inheritdoc */
    override get type(): VmTypeName {
        return 'extern';
    }
    /** @inheritdoc */
    override get tag(): string {
        const tag = getTag(this.value);
        if (this.isArrayLike()) {
            return `${tag}(${this.value.length})`;
        }
        return tag;
    }
}

Object.defineProperty(VmExtern.prototype, kVmExtern, { value: true });
