import type { Writable } from 'type-fest';
import type { DiagnosticCode } from '@mirascript/constants';
import { defineProperty, freeze } from '../../helpers/utils.js';
import { kVmFunction, VM_FUNCTION_ANONYMOUS_NAME } from '../../helpers/constants.js';
import { isVmFunction } from '../../helpers/types.js';
import type { VmAny, VmValue } from './index.js';
import { fromVmFunctionProxy } from './boundary.js';
import type { VmLib } from '../lib/helpers.js';

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
export type VmFunctionInfo = {
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
    /** 标记为弃用 */
    readonly deprecated?: { use?: string; message: DiagnosticCode };
};

/** Mirascript 函数创建选项 */
export type VmFunctionOption = Partial<Omit<VmFunctionInfo, 'original'>> & {
    /** 函数名称 */
    readonly name?: string | null | undefined;
};

const nameIfNotAnonymous = <T>({ name }: { name: string | undefined }, fallback: T): string | T => {
    if (!name) return fallback;
    if (name === VM_FUNCTION_ANONYMOUS_NAME) return fallback;
    return name;
};

/** 创建 Mirascript 函数 */
export function VmFunction<T extends VmFunctionLike>(
    fn: T,
    option: NoInfer<VmFunctionOption | VmFunction | VmLib<T>> = {},
): VmFunction<T> {
    if (typeof fn != 'function') {
        throw new TypeError('Invalid function');
    }

    const exists = fromVmFunctionProxy(fn);
    // 如果已经是 VmFunction，则直接返回
    if (exists) return exists;
    let opt: VmFunctionOption;
    if (isVmFunction(option)) {
        opt = { ...option[kVmFunction], name: nameIfNotAnonymous(option, null) };
    } else if (typeof option == 'function') {
        opt = {
            ...option,
            isLib: true,
            name: nameIfNotAnonymous(option, null),
        };
    } else {
        opt = option;
    }

    const info: Writable<VmFunctionInfo & { __proto__: null }> = {
        __proto__: null,
        fullName: opt.fullName ?? nameIfNotAnonymous(fn, ''),
        isLib: opt.isLib ?? false,
        summary: opt.summary || undefined,
        params: opt.params,
        paramsType: opt.paramsType,
        returns: opt.returns || undefined,
        returnsType: opt.returnsType || undefined,
        examples: opt.examples?.length ? opt.examples : undefined,
        deprecated: opt.deprecated ?? undefined,
    };
    const name = opt.name ?? fn.name;
    defineProperty(fn, 'name', { value: name, configurable: true });
    defineProperty(fn, kVmFunction, { value: freeze(info) });
    return fn as VmFunction<T>;
}
