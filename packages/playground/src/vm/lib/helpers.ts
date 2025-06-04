import { VmError } from '../error';
import { isVmArray, isVmFunction, type VmAny, type VmArray, type VmValue } from '../types';

/** 抛出异常 */
export function throwError(message: string, recovered: VmAny | (() => VmAny)): never {
    const recoveredValue = typeof recovered === 'function' && !isVmFunction(recovered) ? recovered() : recovered;
    throw new VmError(message, recoveredValue);
}

/** 重新抛出异常 */
export function rethrowError(prefix: string, error: unknown, recovered: VmAny | (() => VmAny)): never {
    const recoveredValue = typeof recovered === 'function' && !isVmFunction(recovered) ? recovered() : recovered;
    throw VmError.from(prefix, error, recoveredValue);
}

/** 标记参数为必须项 */
export function required<const T = VmValue>(
    name: string | number,
    value: T | undefined,
    recovered: VmAny | (() => VmAny),
): asserts value is T {
    if (value === undefined) {
        if (typeof name == 'string') throwError(`Missing required parameter: ${name}`, recovered);
        const pos = name <= 0 ? 'first' : name <= 1 ? 'second' : name + 1 + 'th';
        throwError(`Missing required parameter at the ${pos} position`, recovered);
    }
}

/** 标记参数为数组 */
export function arrayRequired(
    name: string | number,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): asserts value is VmArray {
    required(name, value, recovered);
    if (!isVmArray(value)) {
        if (typeof name == 'string') throwError(`Expected array for parameter: ${name}`, recovered);
        const pos = name <= 0 ? 'first' : name <= 1 ? 'second' : name + 1 + 'th';
        throwError(`Expected array at the ${pos} position`, recovered);
    }
}
