import { Cp, Element } from '../helpers';
import { $CallDyn, $ToBoolean, $ToNumber, $ToString } from '../operations';
import {
    isVmArray,
    isVmConst,
    isVmExtern,
    isVmModule,
    isVmPrimitive,
    isVmRecord,
    VmExtern,
    type VmAny,
    type VmArray,
    type VmConst,
    type VmRecord,
    type VmValue,
} from '../types/index.js';
import { VmError } from '../error';
import { expectArray, expectArrayOrRecord, expectCallable, required, rethrowError } from './helpers';
import type { VmLib } from './loader';
const {
    PI,
    E,
    abs,
    acos,
    acosh,
    asin,
    asinh,
    atan,
    atan2,
    atanh,
    cbrt,
    ceil,
    cos,
    cosh,
    exp,
    expm1,
    floor,
    hypot,
    log,
    log10,
    log1p,
    log2,
    pow,
    random,
    round,
    sign,
    sin,
    sinh,
    sqrt,
    tan,
    tanh,
    trunc,
} = Math;

/** Get the minimum and maximum numbers from the arguments. */
function getMinMaxNumbers(args: readonly VmAny[]): number[] {
    if (args.length === 0) return [];
    if (args.length === 1 && isVmArray(args[0])) args = args[0];
    const numbers: number[] = [];
    for (const arg of args) {
        if (arg === undefined) continue;
        numbers.push($ToNumber(arg));
    }
    return numbers;
}

export const max: VmLib = (...args) => {
    const numbers = getMinMaxNumbers(args);
    return Math.max(...numbers);
};
max.summary = '返回一组数中的最大值';
max.params = { '..args': '要比较的数值' };
max.paramsType = { '..args': '[number]' };
max.returnsType = 'number';
export const min: VmLib = (...args) => {
    const numbers = getMinMaxNumbers(args);
    return Math.min(...numbers);
};
min.summary = '返回一组数中的最小值';
min.params = { '..args': '要比较的数值' };
min.paramsType = { '..args': '[number]' };
min.returnsType = 'number';

/** map 和 filter 的实现 */
function mapImpl(
    data: VmAny,
    fnName: string,
    fn: VmAny,
    mapper: (fn: VmValue, value: VmValue, index: number | string | null, data: VmValue) => VmValue | undefined,
): VmValue {
    required('data', data, null);
    expectCallable(fnName, fn, data);
    if (isVmPrimitive(data)) {
        return mapper(fn, data, null, data) ?? null;
    }
    if (isVmArray(data)) {
        const result: VmConst[] = [];
        const { length } = data;
        for (let i = 0; i < length; i++) {
            Cp();
            const ret = mapper(fn, data[i] ?? null, i, data);
            if (ret === undefined) continue;
            if (isVmConst(ret)) {
                result.push(ret);
            } else {
                result.push(null);
            }
        }
        return result;
    }
    if (isVmRecord(data)) {
        const entries: Array<[string, VmConst]> = [];
        for (const [key, value] of Object.entries(data)) {
            Cp();
            const ret = mapper(fn, value, key, data);
            if (ret === undefined) continue;
            if (isVmConst(ret)) {
                entries.push([key, ret]);
            } else {
                entries.push([key, null]);
            }
        }
        return Object.fromEntries(entries);
    }
    if (isVmExtern(data)) {
        if (Array.isArray(data.value)) {
            let isConst = true;
            const result: VmValue[] = [];
            const { length } = data.value;
            for (let i = 0; i < length; i++) {
                Cp();
                const ret = mapper(fn, data.get(String(i)) ?? null, i, data);
                if (ret === undefined) continue;
                if (!isVmConst(ret)) {
                    isConst = false;
                }
                result.push(ret);
            }
            if (isConst) return result as VmArray;
            return new VmExtern(result);
        }
        let isConst = true;
        const result: Array<[string, VmValue]> = [];
        for (const key of data.keys()) {
            Cp();
            const ret = mapper(fn, data.get(key) ?? null, key, data);
            if (ret === undefined) continue;
            if (!isVmConst(ret)) {
                isConst = false;
            }
            result.push([key, ret]);
        }
        const obj = Object.fromEntries(result);
        if (isConst) return obj as VmRecord;
        return new VmExtern(obj);
    }
    throw new VmError('First argument must be primitive, array, record, or extern', null);
}

export const map: VmLib = (data, f) => {
    return mapImpl(data, 'f', f, (fn, value, key, data) => {
        return $CallDyn(fn, [value, key, data]);
    });
};
map.summary = '对数组、记录或外部对象中的每个元素应用函数，并返回结果';
map.params = {
    data: '要映射的数组、记录或外部对象',
    f: '应用于每个元素的函数',
};
map.paramsType = {
    data: 'array | record | extern',
    f: 'fn (value: any, key: number | string | nil, data: type(data)) -> any',
};
map.returnsType = 'type(data)';

export const filter: VmLib = (data, predicate) => {
    return mapImpl(data, 'predicate', predicate, (fn, value, key, data) => {
        const ret = $CallDyn(fn, [value, key, data]);
        return $ToBoolean(ret) ? value : undefined;
    });
};
filter.summary = '过滤数组、记录或外部对象中的元素，返回满足条件的元素';
filter.params = {
    data: '要过滤的数组、记录或外部对象',
    predicate: '用于测试每个元素的函数，返回 true 或 false',
};
filter.paramsType = {
    data: 'array | record | extern',
    predicate: 'fn (value: any, key: number | string | nil, data: type(data)) -> boolean',
};
filter.returnsType = 'type(data)';

export const filter_map: VmLib = (data, f) => {
    return mapImpl(data, 'f', f, (fn, value, key, data) => {
        const ret = $CallDyn(fn, [value, key, data]);
        return ret ?? undefined;
    });
};
filter_map.summary = '对数组、记录或外部对象中的每个元素应用函数，并返回非 nil 的结果';
filter_map.params = {
    data: '要映射的数组、记录或外部对象',
    f: '应用于每个元素的函数，返回 nil 或非 nil 的值',
};
filter_map.paramsType = {
    data: 'array | record | extern',
    f: 'fn (value: any, key: number | string | nil, data: type(data)) -> any | nil',
};
filter_map.returnsType = 'type(data)';

export const len: VmLib = (arr) => {
    expectArray(0, arr, Number.NaN);
    return arr.length;
};
len.summary = '返回数组的长度';
len.params = {
    arr: '要求长度的数组',
};
len.paramsType = {
    arr: 'array',
};
len.returnsType = 'number';

export const chars: VmLib = (str) => {
    required('str', str, null);
    return [...$ToString(str)];
};
chars.summary = '将字符串转换为字符数组';
chars.params = {
    str: '要转换的字符串',
};
chars.paramsType = {
    str: 'string',
};
chars.returnsType = '[string]';

export const to_json: VmLib = (data) => {
    if (isVmExtern(data)) {
        return JSON.stringify(data.value);
    }
    if (isVmModule(data)) {
        return '{}';
    }
    return JSON.stringify(data);
};
to_json.summary = '将数据转换为 JSON 字符串';
to_json.params = {
    data: '要转换为 JSON 的数据',
};
to_json.paramsType = {
    data: 'any',
};
to_json.returnsType = 'string';

export const from_json: VmLib = (json, fallback) => {
    required('json', json, null);
    if (typeof json != 'string') return json;
    try {
        return JSON.parse($ToString(json));
    } catch (ex) {
        rethrowError('Invalid JSON', ex, fallback ?? null);
    }
};
from_json.summary = '将 JSON 字符串转换为数据';
from_json.params = {
    json: '要转换的 JSON 字符串',
    fallback: '如果转换失败，返回的默认值',
};
from_json.paramsType = {
    json: 'string',
    fallback: 'any',
};
from_json.returnsType = 'any';

export const _with_: VmLib = (data, ...entries) => {
    expectArrayOrRecord('data', data, data);
    if (entries.length % 2 !== 0) {
        throw new VmError('Invalid number of arguments, expected even number of arguments', data);
    }
    if (isVmArray(data)) {
        const result: VmConst[] = [...data];
        for (let i = 0; i < entries.length; i += 2) {
            const index = Math.trunc($ToNumber(entries[i]));
            if (!Number.isFinite(index) || index < 0 || index >= Number.MAX_SAFE_INTEGER) continue;
            const value = entries[i + 1];
            while (index > result.length) {
                result.push(null);
            }
            result[index] = Element(value);
        }
        return result;
    } else {
        const result: Record<string, VmConst> = { ...data };
        for (let i = 0; i < entries.length; i += 2) {
            const key = $ToString(entries[i]);
            const value = entries[i + 1];
            result[key] = Element(value);
        }
        return result;
    }
};
_with_.summary = '在数组或记录中设置多个键值对';
_with_.params = {
    data: '要设置的数组或记录',
    '..entries': '要设置的键值对，成对出现',
};
_with_.paramsType = {
    data: 'array | record',
    '..entries': '[..(string | number, any)]',
};
_with_.returnsType = 'type(data)';

export const to_string: VmLib = (data) => $ToString(data);

export const to_number: VmLib = (data) => $ToNumber(data);

export const to_boolean: VmLib = (data) => $ToBoolean(data);

export const _abs_: VmLib = (value) => abs($ToNumber(value));
export const _acos_: VmLib = (value) => acos($ToNumber(value));
export const _acosh_: VmLib = (value) => acosh($ToNumber(value));
export const _asin_: VmLib = (value) => asin($ToNumber(value));
export const _asinh_: VmLib = (value) => asinh($ToNumber(value));
export const _atan_: VmLib = (value) => atan($ToNumber(value));
export const _atan2_: VmLib = (x, y) => atan2($ToNumber(x), $ToNumber(y));
export const _atanh_: VmLib = (value) => atanh($ToNumber(value));
export const _cbrt_: VmLib = (value) => cbrt($ToNumber(value));
export const _ceil_: VmLib = (value) => ceil($ToNumber(value));
export const _cos_: VmLib = (value) => cos($ToNumber(value));
export const _cosh_: VmLib = (value) => cosh($ToNumber(value));
export const _exp_: VmLib = (value) => exp($ToNumber(value));
export const _expm1_: VmLib = (value) => expm1($ToNumber(value));
export const _floor_: VmLib = (value) => floor($ToNumber(value));
export const _hypot_: VmLib = (...values) => hypot(...values.map($ToNumber));
export const _log_: VmLib = (value) => log($ToNumber(value));
export const _log10_: VmLib = (value) => log10($ToNumber(value));
export const _log1p_: VmLib = (value) => log1p($ToNumber(value));
export const _log2_: VmLib = (value) => log2($ToNumber(value));
export const _pow_: VmLib = (x, y) => pow($ToNumber(x), $ToNumber(y));
export const _random_: VmLib = () => random();
export const _round_: VmLib = (value) => round($ToNumber(value));
export const _sign_: VmLib = (value) => sign($ToNumber(value));
export const _sin_: VmLib = (value) => sin($ToNumber(value));
export const _sinh_: VmLib = (value) => sinh($ToNumber(value));
export const _sqrt_: VmLib = (value) => sqrt($ToNumber(value));
export const _tan_: VmLib = (value) => tan($ToNumber(value));
export const _tanh_: VmLib = (value) => tanh($ToNumber(value));
export const _trunc_: VmLib = (value) => trunc($ToNumber(value));

export { PI as '@pi', E as '@e' };
