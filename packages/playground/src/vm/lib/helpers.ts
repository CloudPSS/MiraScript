import { VmError } from '../error.js';
import { $Type } from '../operations.js';
import {
    isVmArray,
    isVmExtern,
    isVmFunction,
    type VmExtern,
    type VmFunction,
    type VmAny,
    type VmArray,
    type VmValue,
    isVmRecord,
    type VmRecord,
} from '../types/index.js';

/** 抛出异常 */
export function throwError(message: string, recovered: VmAny | (() => VmAny)): never {
    const recoveredValue = typeof recovered === 'function' && !isVmFunction(recovered) ? recovered() : recovered;
    throw new VmError(message, recoveredValue);
}

/** 抛出预期外类型异常 */
export function throwUnexpectedTypeError(
    name: string | number,
    expected: string,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): never {
    const actual = $Type(value);
    if (typeof name == 'string') throwError(`Expected ${expected} for parameter '${name}', got ${actual}`, recovered);
    const pos = name <= 0 ? 'first' : name <= 1 ? 'second' : name + 1 + 'th';
    throwError(`Expected ${expected} at the ${pos} position, got ${actual}`, recovered);
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
        if (typeof name == 'string') throwError(`Missing required parameter '${name}'`, recovered);
        const pos = name <= 0 ? 'first' : name <= 1 ? 'second' : name + 1 + 'th';
        throwError(`Missing required parameter at the ${pos} position`, recovered);
    }
}

/** 标记参数为数组 */
export function expectArray(
    name: string | number,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): asserts value is VmArray {
    required(name, value, recovered);
    if (!isVmArray(value)) {
        throwUnexpectedTypeError(name, 'array', value, recovered);
    }
}

/** 标记参数为记录 */
export function expectRecord(
    name: string | number,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): asserts value is VmRecord {
    required(name, value, recovered);
    if (!isVmRecord(value)) {
        throwUnexpectedTypeError(name, 'record', value, recovered);
    }
}

/** 标记参数为数组或记录 */
export function expectArrayOrRecord(
    name: string | number,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): asserts value is VmArray | VmRecord {
    required(name, value, recovered);
    if (!isVmArray(value) && !isVmRecord(value)) {
        throwUnexpectedTypeError(name, 'array or record', value, recovered);
    }
}

/** 标记为可调用 */
export function expectCallable(
    name: string | number,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): asserts value is VmFunction | VmExtern {
    required(name, value, recovered);
    const callable = isVmFunction(value) || (isVmExtern(value) && value.callable);
    if (!callable) {
        throwUnexpectedTypeError(name, 'callable', value, recovered);
    }
}
