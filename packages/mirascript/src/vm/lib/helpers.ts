import { VM_ARRAY_MAX_LENGTH } from '../../helpers/constants.js';
import { isNaN, NotNumber, entries, fromEntries, isSafeInteger, isFinite } from '../../helpers/utils.js';
import { toBoolean, toNumber, toString } from '../../helpers/convert/index.js';
import { display } from '../../helpers/serialize.js';
import { isVmArray, isVmFunction, isVmPrimitive, isVmConst, isVmCallable, isVmRecord } from '../../helpers/types.js';
import { VmError } from '../../helpers/error.js';
import type {
    VmExtern,
    VmFunction,
    VmAny,
    VmArray,
    VmValue,
    VmRecord,
    VmModule,
    VmConst,
    VmFunctionLike,
    VmFunctionOption,
} from '../types/index.js';
import { Cp } from '../checkpoint.js';

/** 抛出异常 */
export function throwError(message: string, recovered: VmAny | (() => VmAny)): never {
    const recoveredValue = typeof recovered === 'function' && !isVmFunction(recovered) ? recovered() : recovered;
    throw new VmError(message, recoveredValue);
}
/** 描述参数 */
type ParamIndex = number | string | null;
/** 描述参数 */
export function describeParam(name: ParamIndex): string {
    if (name == null) return 'Value';
    if (typeof name == 'string') {
        if (!name) return 'Argument';
        return `Argument '${name}'`;
    }
    const pos = name <= 0 ? 'first' : name <= 1 ? 'second' : `${name + 1}th`;
    return `Argument at the ${pos} position`;
}

/** 抛出预期外类型异常 */
export function throwUnexpectedTypeError(
    name: ParamIndex,
    expected: string,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): never {
    throwError(`${describeParam(name)} is not ${expected}: ${display(value)}`, recovered);
}
/** 抛出预期外类型异常 */
export function throwUnconvertedTypeError(
    name: ParamIndex,
    expected: string,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): never {
    throwError(`${describeParam(name)} cannot be converted to ${expected}: ${display(value)}`, recovered);
}

/** 重新抛出异常 */
export function rethrowError(prefix: string, error: unknown, recovered: VmAny | (() => VmAny)): never {
    const recoveredValue = typeof recovered === 'function' && !isVmFunction(recovered) ? recovered() : recovered;
    throw VmError.from(prefix, error, recoveredValue);
}

/** 标记参数为必须项 */
export function required<const T = VmValue>(
    name: ParamIndex,
    value: T | undefined,
    recovered: VmAny | (() => VmAny),
): asserts value is T {
    if (value === undefined) {
        throwError(`${describeParam(name)} is required`, recovered);
    }
}

/** 标记并转换参数为数字 */
export function expectNumber(name: ParamIndex, value: VmAny): number {
    required(name, value, NotNumber);
    const v = toNumber(value, null);
    if (v == null) {
        throwUnconvertedTypeError(name, 'number', value, NotNumber);
    }
    return v;
}
/** 标记并转换参数为数字 */
export function expectNumberRange(name: ParamIndex, value: VmAny, min: number, max: number): number {
    const v = expectNumber(name, value);
    if (!isFinite(v)) {
        throwError(`${describeParam(name)} is not a finite number: ${display(value)}`, NotNumber);
    }
    if (v < min) {
        throwError(`${describeParam(name)} is less than minimum value ${min}: ${display(value)}`, min);
    }
    if (v > max) {
        throwError(`${describeParam(name)} is greater than maximum value ${max}: ${display(value)}`, max);
    }
    return v;
}
/** 标记并转换参数为整数 */
export function expectIntegerRange(name: ParamIndex, value: VmAny, min: number, max: number): number {
    const i = expectInteger(name, value);
    if (i < min) {
        throwError(`${describeParam(name)} is less than minimum value ${min}: ${display(value)}`, min);
    }
    if (i > max) {
        throwError(`${describeParam(name)} is greater than maximum value ${max}: ${display(value)}`, max);
    }
    return i;
}
/** 标记并转换参数为整数 */
export function expectInteger(name: ParamIndex, value: VmAny): number {
    required(name, value, 0);
    const v = toNumber(value, null);
    if (v == null) {
        throwUnconvertedTypeError(name, 'integer', value, 0);
    }
    const i = Math.trunc(v);
    if (!isSafeInteger(i)) {
        throwUnconvertedTypeError(name, 'integer', value, 0);
    }
    return i;
}

/** 标记并转换参数为布尔值 */
export function expectBoolean(name: ParamIndex, value: VmAny): boolean {
    required(name, value, false);
    const v = toBoolean(value, null);
    if (v == null) {
        throwUnconvertedTypeError(name, 'boolean', value, false);
    }
    return v;
}

/** 标记并转换参数为字符串 */
export function expectString(name: ParamIndex, value: VmAny): string {
    required(name, value, '');
    const v = toString(value, null);
    if (v == null) {
        throwUnconvertedTypeError(name, 'string', value, '');
    }
    return v;
}

/** 标记参数为数组 */
export function expectArray(
    name: ParamIndex,
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
    name: ParamIndex,
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
    name: ParamIndex,
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
    name: ParamIndex,
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
    name: ParamIndex,
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
    name: ParamIndex,
    value: VmAny,
    recovered: VmAny | (() => VmAny),
): asserts value is VmFunction | VmExtern {
    required(name, value, recovered);
    if (!isVmCallable(value)) {
        throwUnexpectedTypeError(name, 'callable', value, recovered);
    }
}

/** Get numbers from the arguments. */
export function getNumbers(args: readonly VmAny[]): number[] {
    if (args.length === 0) return [];
    let useFirst = false;
    if (args.length === 1 && isVmArray(args[0])) {
        args = args[0];
        useFirst = true;
    }
    const numbers: number[] = [];
    for (let len = args.length, i = 0; i < len; i++) {
        numbers.push(expectNumber(useFirst ? null : i, args[i]));
    }
    return numbers;
}

/** 将值转为数组长度 */
export function arrayLen(len: number | null | undefined): number {
    if (len == null || isNaN(len) || len <= -1) {
        throwError('Array length must be a non-negative integer', null);
    }
    len = Math.trunc(len);
    if (len > VM_ARRAY_MAX_LENGTH) {
        throwError(`Array length exceeds maximum limit of ${VM_ARRAY_MAX_LENGTH}`, null);
    }
    return len;
}

/** 应用映射函数 */
export function map(
    data: VmConst,
    /** 返回 `undefined` 表示跳过该元素 */
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
            result.push(ret);
        }
        return result;
    } else {
        const e: Array<[string, VmConst]> = [];
        for (const [key, value] of entries(data)) {
            Cp();
            const ret = mapper(value ?? null, key, data);
            if (ret === undefined) continue;
            e.push([key, ret]);
        }
        return fromEntries(e);
    }
}

/** 库函数选项 */
export type VmLibOption = Pick<VmFunctionOption, 'summary' | 'params' | 'returns' | 'examples' | 'deprecated'>;
/** 库函数 */
export type VmLib<T extends VmFunctionLike | VmConst = VmFunctionLike> = (T extends VmFunctionLike ? T : { value: T }) &
    VmLibOption;

/** 创建库函数 */
export function VmLib<
    const T extends VmFunctionLike | VmConst,
    P extends Record<string, unknown> = Record<never, never>,
>(value: T, option: VmLibOption, properties?: P): VmLib<T> & P {
    if (isVmFunction(value)) throw new TypeError('Cannot create VmLib from a VmFunction');

    // 后续在 wrapEntry 中会处理函数的包装
    const ret = (typeof value == 'function' ? value : { __proto__: null, value: value }) as VmLib<T> & P;
    Object.defineProperties(ret, {
        summary: { enumerable: true, value: option.summary },
        params: { enumerable: true, value: option.params },
        returns: { enumerable: true, value: option.returns },
        examples: { enumerable: true, value: option.examples },
        deprecated: { enumerable: true, value: option.deprecated ?? undefined },
    });
    Object.assign(ret, properties);
    return ret;
}
