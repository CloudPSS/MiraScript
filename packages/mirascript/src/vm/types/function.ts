import type { Writable } from 'type-fest';
import { defineProperty } from '../../helpers/utils.js';
import { CpEnter, CpExit } from '../helpers.js';
import { $Call } from '../operations.js';
import type { VmAny, VmValue } from './index.js';
import { wrapToVmValue, unwrapFromVmValue } from './extern.js';

const kVmFunction = Symbol.for('mirascript.vm.function');
const kProxy = Symbol.for('mirascript.vm.function.proxy');

/**
 * Mirascript 函数签名
 *
 * 虽然所有输入参数的类型均为 {@linkcode VmValue}，但当参数不足时，对应的参数会被填充为 `undefined`。
 */
// eslint-disable-next-line @typescript-eslint/no-invalid-void-type
export type VmFunctionLike = (...args: ReadonlyArray<VmValue | undefined>) => VmAny | void;

/** Mirascript 函数 */
export type VmFunction<T extends VmFunctionLike = VmFunctionLike> = T & { readonly [kVmFunction]: VmFunctionInfo };

/** Mirascript 函数信息 */
export interface VmFunctionInfo {
    /** 完整名称 */
    readonly fullName: string;
    /** 是否为库函数 */
    readonly isLib: boolean;
    /** 文档字符串 */
    readonly summary?: string;
    /** 文档字符串 */
    readonly params?: Record<string, string>;
    /** 文档字符串 */
    readonly paramsType?: Record<string, string>;
    /** 文档字符串 */
    readonly returns?: string;
    /** 文档字符串 */
    readonly returnsType?: string;
    /** 文档字符串 */
    readonly examples?: string[];
    /** 如果添加了包装，返回原函数 */
    readonly original?: VmFunctionLike;
}

/** Mirascript 函数创建选项 */
export type VmFunctionOption = Partial<
    Omit<VmFunctionInfo, 'original'> & {
        readonly injectCp: boolean;
    }
>;

/** 检查是否为 Mirascript 函数 */
export function isVmFunction<T extends VmFunctionLike>(value: unknown): value is VmFunction<T> {
    return getVmFunctionInfo(value) != null;
}

/** 检查是否为 Mirascript 函数，并获取其信息 */
export function getVmFunctionInfo(value: unknown): VmFunctionInfo | undefined {
    if (typeof value != 'function') return undefined;
    return (value as VmFunction)[kVmFunction];
}

/** 创建 Mirascript 函数 */
export function VmFunction<T extends VmFunctionLike>(fn: T, option: VmFunctionOption = {}): VmFunction<T> {
    if (typeof fn != 'function') {
        throw new TypeError('Invalid function');
    }

    const exists = fromVmFunctionProxy(fn);
    // 如果已经是 VmFunction，则直接返回
    if (exists) return exists;

    const info: Writable<VmFunctionInfo> = {
        fullName: option.fullName ?? fn.name,
        isLib: option.isLib ?? false,
        summary: option.summary || undefined,
        params: option.params,
        paramsType: option.paramsType,
        returns: option.returns || undefined,
        returnsType: option.returnsType || undefined,
        examples: option.examples?.length ? option.examples : undefined,
    };
    if (option.injectCp) {
        const original = fn;
        info.original = original;
        fn = ((...args) => {
            try {
                CpEnter();
                const ret = original(...args);
                return ret;
            } finally {
                CpExit();
            }
        }) as typeof fn;
        defineProperty(fn, 'name', {
            value: original.name,
            configurable: true,
        });
    }
    defineProperty(fn, kVmFunction, {
        value: Object.freeze(info),
    });
    return fn as VmFunction<T>;
}

/** 创建 Mirascript 函数在宿主语言运行的代理 */
export function toVmFunctionProxy<T extends VmFunctionLike>(fn: VmFunction<T>): T {
    if (!isVmFunction(fn)) return fn;

    const cached = (fn as unknown as { [kProxy]?: T })[kProxy];
    if (cached) return cached;

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
