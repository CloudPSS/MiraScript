import { defineProperty, apply } from '../../helpers/utils.js';
import { isVmExtern, isVmFunction, isVmWrapper } from '../../helpers/types.js';
import { kVmFunctionProxy } from '../../helpers/constants.js';
import type { VmFunctionLike, VmFunction } from './function.js';
import { VmExtern } from './extern.js';
import type { VmAny, VmConst, VmModule, VmPrimitive, VmValue } from './index.js';
import { $Call } from '../operations/call.js';

/** 创建 Mirascript 函数在宿主语言运行的代理 */
export function toVmFunctionProxy<T extends VmFunctionLike>(fn: VmFunction<T>): T {
    const cached = (fn as unknown as { [kVmFunctionProxy]?: T })[kVmFunctionProxy];
    if (cached != null) return cached;

    const proxy = (...args: unknown[]) => {
        const ret = $Call(
            fn,
            // 作为函数参数传入的值一定丢失了它的 this
            args.map((v) => wrapToVmValue(v, null, null)),
        );
        return unwrapFromVmValue(ret, true);
    };
    defineProperty(fn, kVmFunctionProxy, { value: proxy });
    defineProperty(proxy, kVmFunctionProxy, { value: fn });
    defineProperty(proxy, 'name', {
        value: fn.name,
        configurable: true,
    });
    return proxy as T;
}

/** 解开 Mirascript 函数在宿主语言运行的代理 */
export function fromVmFunctionProxy<T extends VmFunctionLike>(fn: T): VmFunction<T> | null {
    if (isVmFunction(fn)) return fn;

    const original = (fn as unknown as { [kVmFunctionProxy]?: VmFunction<T> })[kVmFunctionProxy];
    if (original && isVmFunction(original)) return original;

    return null;
}

/** 将宿主语言的值包装为 Mirascript 类型 */
export function wrapToVmValue(
    value: unknown,
    thisArg: unknown = null,
    assumeVmValue: ((obj: object) => obj is Exclude<VmConst, VmPrimitive>) | null = null,
): VmValue {
    if (value == null) return null;
    switch (typeof value) {
        case 'function': {
            const unwrapped = fromVmFunctionProxy(value as VmFunctionLike);
            if (unwrapped != null) return unwrapped;
            return new VmExtern(value as () => never, thisArg);
        }
        case 'object': {
            if (isVmWrapper(value)) return value as VmModule | VmExtern;
            if (value instanceof Date) return value.valueOf();
            if (assumeVmValue?.(value)) return value;
            // Only functions preserve thisArg
            return new VmExtern(value);
        }
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

/** 创建绑定 */
function bindThis<T extends (...args: unknown[]) => unknown>(fn: T, thisArg: unknown): T {
    if (thisArg == null) return fn;
    return new Proxy(fn, {
        apply(target, _thisArg, args) {
            return apply(target, thisArg, args);
        },
    });
}

/** 取消宿主语言的值的 Mirascript 包装  */
export function unwrapFromVmValue(value: VmAny, bindThisArg = true): unknown {
    if (isVmFunction(value)) {
        return toVmFunctionProxy(value);
    }
    if (value == null || typeof value != 'object') return value;
    if (!isVmExtern(value)) return value;

    if (value.thisArg == null || typeof value.value != 'function') {
        return value.value;
    }
    const f = value as VmExtern<(...args: unknown[]) => unknown>;
    return bindThisArg ? bindThis(f.value, f.thisArg) : f.value;
}
