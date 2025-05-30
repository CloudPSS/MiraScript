import { $Cp } from './operations.js';
import type { VmValue } from './types.js';

const kVmFunction = Symbol.for('mirascript.vm.function');

/** Mirascript 函数 */
export type VmFunction = { (...args: VmValue[]): VmValue; [kVmFunction]: VmFunctionInfo };

/** Mirascript 函数信息 */
export interface VmFunctionInfo {
    /** 完整名称 */
    readonly fullName: string;
    /** 是否为库函数 */
    readonly isLib: boolean;
}

/** 检查是否为 Mirascript 函数 */
export function isVmFunction(value: unknown): value is VmFunction {
    return typeof value === 'function' && kVmFunction in (value as VmFunction);
}

/** 创建 Mirascript 函数 */
export function VmFunction(info: VmFunctionInfo, fn: (...args: VmValue[]) => VmValue): VmFunction {
    const wrapped = ((...args: VmValue[]) => {
        $Cp();
        const result = fn(...args);
        return result ?? null;
    }) as VmFunction;
    Object.defineProperty(wrapped, kVmFunction, {
        value: { isLib: info.isLib },
    });
    Object.defineProperty(wrapped, 'name', {
        value: fn.name,
    });
    return wrapped;
}
