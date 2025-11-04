import { isVmFunction, type VmFunctionLike, type VmFunction } from './function.js';
import { isVmExtern, VmExtern } from './extern.js';
import { isVmWrapper } from './wrapper.js';
import type { VmAny, VmConst, VmModule, VmPrimitive, VmValue } from './index.js';
import { $Call } from '../operations.js';
import { defineProperty, apply } from '../../helpers/utils.js';

const kProxy = Symbol.for('mirascript.vm.function.proxy');

/** 创建 Mirascript 函数在宿主语言运行的代理 */
export function toVmFunctionProxy<T extends VmFunctionLike>(fn: VmFunction<T>): T {
    if (!isVmFunction(fn)) return fn;

    const cached = (fn as unknown as { [kProxy]?: T })[kProxy];
    if (cached != null) return cached;

    const proxy = (...args: unknown[]) => {
        const ret = $Call(
            fn,
            // 作为函数参数传入的值一定丢失了它的 this
            args.map((v) => wrapToVmValue(v, null)),
        );
        return unwrapFromVmValue(ret);
    };
    defineProperty(fn, kProxy, { value: proxy });
    defineProperty(proxy, kProxy, { value: fn });
    defineProperty(proxy, 'name', {
        value: fn.name,
        configurable: true,
    });
    return proxy as T;
}

/** 解开 Mirascript 函数在宿主语言运行的代理 */
export function fromVmFunctionProxy<T extends VmFunctionLike>(fn: T): VmFunction<T> | undefined {
    if (isVmFunction(fn)) return fn;

    const original = (fn as unknown as { [kProxy]?: VmFunction<T> })[kProxy];
    if (original && isVmFunction(original)) return original;

    return undefined;
}

/** 将宿主语言的值包装为 Mirascript 类型 */
export function wrapToVmValue(
    value: unknown,
    thisArg: VmExtern | null = null,
    assumeVmValue?: (obj: object) => obj is Exclude<VmConst, VmPrimitive>,
): VmValue {
    if (value == null) return null;
    switch (typeof value) {
        case 'function': {
            const unwrapped = fromVmFunctionProxy(value as VmFunctionLike);
            if (unwrapped) return unwrapped;
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

/** 取消宿主语言的值的 Mirascript 包装  */
export function unwrapFromVmValue(value: VmAny): unknown {
    if (typeof value == 'function') {
        return toVmFunctionProxy(value);
    }
    if (value == null || typeof value != 'object') return value;
    if (!isVmExtern(value)) return value;

    if (value.thisArg == null || typeof value.value != 'function') {
        return value.value;
    }
    const f = value as VmExtern<(...args: unknown[]) => unknown>;
    const caller = f.thisArg!.value;
    return new Proxy(f.value, {
        apply(target, thisArg, args): unknown {
            return apply(target, caller, args);
        },
    });
}
