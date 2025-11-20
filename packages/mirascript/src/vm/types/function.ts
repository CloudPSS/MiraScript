import type { Writable } from 'type-fest';
import { defineProperty } from '../../helpers/utils.js';
import { kVmFunction, VM_FUNCTION_ANONYMOUS_NAME } from '../../helpers/constants.js';
import type { VmAny, VmValue } from './index.js';
import { fromVmFunctionProxy } from './boundary.js';
import { CpEnter, CpExit } from '../checkpoint.js';

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

/** 创建 Mirascript 函数 */
export function VmFunction<T extends VmFunctionLike>(fn: T, option: VmFunctionOption = {}): VmFunction<T> {
    if (typeof fn != 'function') {
        throw new TypeError('Invalid function');
    }

    const exists = fromVmFunctionProxy(fn);
    // 如果已经是 VmFunction，则直接返回
    if (exists) return exists;

    const info: Writable<VmFunctionInfo> = {
        fullName: option.fullName ?? (fn.name === VM_FUNCTION_ANONYMOUS_NAME ? '' : fn.name),
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
        defineProperty(fn, 'name', { value: original.name });
    }
    defineProperty(fn, kVmFunction, {
        value: Object.freeze(info),
    });
    return fn as VmFunction<T>;
}
