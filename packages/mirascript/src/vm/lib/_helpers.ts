import { VmError } from '../error.js';
import { $ToNumber, $Type } from '../operations.js';
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
    type VmModule,
    isVmPrimitive,
    type VmConst,
    isVmConst,
    VM_ARRAY_MAX_LENGTH,
} from '../types/index.js';
import type { VmFunctionLike, VmFunctionOption } from '../types/function.js';
import { Cp } from '../helpers.js';
import { isNaN, entries, fromEntries } from '../../helpers/utils.js';

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
        throwUnexpectedTypeError(name, 'array | record', value, recovered);
    }
}

/** 标记参数为复合类型 */
export function expectCompound(
    name: string | number,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): asserts value is VmArray | VmRecord | VmModule | VmExtern {
    required(name, value, recovered);
    if (isVmPrimitive(value) || isVmFunction(value)) {
        throwUnexpectedTypeError(name, 'array | record | module | extern', value, recovered);
    }
}

/** 标记参数为常量 */
export function expectConst(
    name: string | number,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): asserts value is VmConst {
    required(name, value, recovered);
    if (!isVmConst(value)) {
        throwUnexpectedTypeError(name, 'nil | number | boolean | string | array | record', value, recovered);
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

/** Get numbers from the arguments. */
export function getNumbers(args: readonly VmAny[]): number[] {
    if (args.length === 0) return [];
    if (args.length === 1 && isVmArray(args[0])) args = args[0];
    const numbers: number[] = [];
    for (const arg of args) {
        if (arg == null) continue;
        numbers.push($ToNumber(arg));
    }
    return numbers;
}

/** 将值转为数组长度 */
export function arrayLen(len: number | null | undefined): number {
    if (len == null || isNaN(len) || len <= 0) return 0;
    len = Math.trunc(len);
    if (len > VM_ARRAY_MAX_LENGTH) throwError(`Array length exceeds maximum limit of ${VM_ARRAY_MAX_LENGTH}`, null);
    return len;
}

/** 应用映射函数 */
export function map(
    data: VmConst,
    mapper: (value: VmConst, index: number | string | null, data: VmConst) => VmConst | undefined,
): VmConst {
    if (isVmPrimitive(data)) {
        return mapper(data, null, data) ?? null;
    }
    if (isVmArray(data)) {
        const result: VmConst[] = [];
        const { length } = data;
        for (let i = 0; i < length; i++) {
            Cp();
            const ret = mapper(data[i] ?? null, i, data);
            if (ret === undefined) continue;
            if (isVmConst(ret)) {
                result.push(ret);
            } else {
                result.push(null);
            }
        }
        return result;
    } else {
        const e: Array<[string, VmConst]> = [];
        for (const [key, value] of entries(data)) {
            Cp();
            const ret = mapper(value ?? null, key, data);
            if (ret === undefined) continue;
            if (isVmConst(ret)) {
                e.push([key, ret]);
            } else {
                e.push([key, null]);
            }
        }
        return fromEntries(e);
    }
}

/** 库函数选项 */
export type VmLibOption = Pick<VmFunctionOption, 'summary' | 'params' | 'paramsType' | 'returns' | 'returnsType'>;
/** 库函数 */
export type VmLib<T extends VmFunctionLike = VmFunctionLike> = T & VmLibOption;

/** 创建库函数 */
export function VmLib<T extends VmFunctionLike>(fn: T, option: VmLibOption): VmLib<T> {
    /* c8 ignore next 2 */
    if (typeof fn != 'function') throw new TypeError('Invalid function');
    if (isVmFunction(fn)) throw new TypeError('Cannot create VmLib from a VmFunction');

    const ret = fn as T & VmLibOption as Writable<VmLibOption>;
    ret.params = option.params;
    ret.paramsType = option.paramsType;
    ret.returns = option.returns;
    ret.returnsType = option.returnsType;
    ret.summary = option.summary;
    return ret as T & VmLibOption;
}
