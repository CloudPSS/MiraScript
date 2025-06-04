import { CpEnter, CpExit } from '../helpers.js';
import type { VmAny, VmValue } from './index.js';

const kVmFunction = Symbol.for('mirascript.vm.function');

/**
 * Mirascript 函数签名
 *
 * 虽然所有输入参数的类型均为 {@linkcode VmValue}，但当参数不足时，对应的参数会被填充为 `undefined`。
 */
export type VmFunctionLike = (...args: ReadonlyArray<VmValue | undefined>) => VmAny;

/** Mirascript 函数 */
export type VmFunction = VmFunctionLike & { readonly [kVmFunction]: VmFunctionInfo };

/** Mirascript 函数信息 */
export interface VmFunctionInfo {
    /** 完整名称 */
    readonly fullName: string;
    /** 是否为库函数 */
    readonly isLib: boolean;
    /** 如果添加了包装，返回原函数 */
    readonly original?: (...args: VmAny[]) => VmAny;
}

/** Mirascript 函数创建选项 */
export type VmFunctionOption = Partial<
    Omit<VmFunctionInfo, 'original'> & {
        readonly injectCp: boolean;
    }
>;

/** 检查是否为 Mirascript 函数 */
export function isVmFunction(value: unknown): value is VmFunction {
    return getVmFunctionInfo(value) != null;
}

/** 检查是否为 Mirascript 函数，并获取其信息 */
export function getVmFunctionInfo(value: unknown): VmFunctionInfo | undefined {
    if (typeof value != 'function') return undefined;
    return (value as VmFunction)[kVmFunction];
}

/** 创建 Mirascript 函数 */
export function VmFunction(fn: VmFunctionLike, option: VmFunctionOption = {}): VmFunction {
    if (typeof fn != 'function') throw new TypeError('Invalid function');
    if (isVmFunction(fn)) {
        // 如果已经是 VmFunction，则直接返回
        return fn;
    }
    const info: Writable<VmFunctionInfo> = {
        fullName: option.fullName ?? fn.name,
        isLib: option.isLib ?? false,
    };
    if (option.injectCp) {
        const original = fn;
        info.original = original;
        fn = ((...args) => {
            CpEnter();
            try {
                const ret = original(...args);
                return ret;
            } finally {
                CpExit();
            }
        }) as typeof fn;
        Object.defineProperty(fn, 'name', {
            value: fn.name,
            configurable: true,
        });
    }
    Object.defineProperty(fn, kVmFunction, {
        value: Object.freeze(info),
    });
    return fn as VmFunction;
}
