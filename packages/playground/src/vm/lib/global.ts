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
    f: 'fn(value: any, key: number | string | nil, data: type(data)) -> any',
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
    predicate: 'fn(value: any, key: number | string | nil, data: type(data)) -> boolean',
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
    f: 'fn(value: any, key: number | string | nil, data: type(data)) -> any | nil',
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
    '..entries': '[string | number | any]',
};
_with_.returnsType = 'type(data)';

export const to_string: VmLib = (data) => $ToString(data);
to_string.summary = '将数据转换为字符串';
to_string.params = { data: '要转换的数据' };
to_string.paramsType = { data: 'any' };
to_string.returnsType = 'string';

export const to_number: VmLib = (data) => $ToNumber(data);
to_number.summary = '将数据转换为数字';
to_number.params = { data: '要转换的数据' };
to_number.paramsType = { data: 'any' };
to_number.returnsType = 'number';

export const to_boolean: VmLib = (data) => $ToBoolean(data);
to_boolean.summary = '将数据转换为布尔值';
to_boolean.params = { data: '要转换的数据' };
to_boolean.paramsType = { data: 'any' };
to_boolean.returnsType = 'boolean';

export const _abs_: VmLib = (x) => abs($ToNumber(x));
_abs_.summary = '返回数值的绝对值';
_abs_.params = { x: '要取绝对值的数' };
_abs_.paramsType = { x: 'number' };
_abs_.returnsType = 'number';

export const _acos_: VmLib = (x) => acos($ToNumber(x));
_acos_.summary = '返回数值的反余弦值（弧度）';
_acos_.params = { x: '要计算反余弦的数' };
_acos_.paramsType = { x: 'number' };
_acos_.returnsType = 'number';

export const _acosh_: VmLib = (x) => acosh($ToNumber(x));
_acosh_.summary = '返回数值的反双曲余弦值';
_acosh_.params = { x: '要计算反双曲余弦的数' };
_acosh_.paramsType = { x: 'number' };
_acosh_.returnsType = 'number';

export const _asin_: VmLib = (x) => asin($ToNumber(x));
_asin_.summary = '返回数值的反正弦值（弧度）';
_asin_.params = { x: '要计算反正弦的数' };
_asin_.paramsType = { x: 'number' };
_asin_.returnsType = 'number';

export const _asinh_: VmLib = (x) => asinh($ToNumber(x));
_asinh_.summary = '返回数值的反双曲正弦值';
_asinh_.params = { x: '要计算反双曲正弦的数' };
_asinh_.paramsType = { x: 'number' };
_asinh_.returnsType = 'number';

export const _atan_: VmLib = (x) => atan($ToNumber(x));
_atan_.summary = '返回数值的反正切值（弧度）';
_atan_.params = { x: '要计算反正切的数' };
_atan_.paramsType = { x: 'number' };
_atan_.returnsType = 'number';

export const _atan2_: VmLib = (x, y) => atan2($ToNumber(x), $ToNumber(y));
_atan2_.summary = '返回从原点到点 (x, y) 的角度（弧度）';
_atan2_.params = { x: 'x 坐标', y: 'y 坐标' };
_atan2_.paramsType = { x: 'number', y: 'number' };
_atan2_.returnsType = 'number';

export const _atanh_: VmLib = (x) => atanh($ToNumber(x));
_atanh_.summary = '返回数值的反双曲正切值';
_atanh_.params = { x: '要计算反双曲正切的数' };
_atanh_.paramsType = { x: 'number' };
_atanh_.returnsType = 'number';

export const _cbrt_: VmLib = (x) => cbrt($ToNumber(x));
_cbrt_.summary = '返回数值的立方根';
_cbrt_.params = { x: '要计算立方根的数' };
_cbrt_.paramsType = { x: 'number' };
_cbrt_.returnsType = 'number';

export const _ceil_: VmLib = (x) => ceil($ToNumber(x));
_ceil_.summary = '返回大于等于给定数的最小整数';
_ceil_.params = { x: '要向上取整的数' };
_ceil_.paramsType = { x: 'number' };
_ceil_.returnsType = 'number';

export const _cos_: VmLib = (x) => cos($ToNumber(x));
_cos_.summary = '返回数值的余弦值';
_cos_.params = { x: '要计算余弦的数（弧度）' };
_cos_.paramsType = { x: 'number' };
_cos_.returnsType = 'number';

export const _cosh_: VmLib = (x) => cosh($ToNumber(x));
_cosh_.summary = '返回数值的双曲余弦值';
_cosh_.params = { x: '要计算双曲余弦的数' };
_cosh_.paramsType = { x: 'number' };
_cosh_.returnsType = 'number';

export const _exp_: VmLib = (x) => exp($ToNumber(x));
_exp_.summary = '返回 e 的指定次幂';
_exp_.params = { x: '指数' };
_exp_.paramsType = { x: 'number' };
_exp_.returnsType = 'number';

export const _expm1_: VmLib = (x) => expm1($ToNumber(x));
_expm1_.summary = '返回 e 的 x 次幂减 1';
_expm1_.params = { x: '指数' };
_expm1_.paramsType = { x: 'number' };
_expm1_.returnsType = 'number';

export const _floor_: VmLib = (x) => floor($ToNumber(x));
_floor_.summary = '返回小于等于给定数的最大整数';
_floor_.params = { x: '要向下取整的数' };
_floor_.paramsType = { x: 'number' };
_floor_.returnsType = 'number';

export const _hypot_: VmLib = (...values) => hypot(...values.map($ToNumber));
_hypot_.summary = '返回所有参数平方和的平方根';
_hypot_.params = { '..values': '要计算的数值' };
_hypot_.paramsType = { '..values': '[number]' };
_hypot_.returnsType = 'number';

export const _log_: VmLib = (x) => log($ToNumber(x));
_log_.summary = '返回数值的自然对数（以 e 为底）';
_log_.params = { x: '要取对数的数' };
_log_.paramsType = { x: 'number' };
_log_.returnsType = 'number';

export const _log10_: VmLib = (x) => log10($ToNumber(x));
_log10_.summary = '返回数值的以 10 为底的对数';
_log10_.params = { x: '要取对数的数' };
_log10_.paramsType = { x: 'number' };
_log10_.returnsType = 'number';

export const _log1p_: VmLib = (x) => log1p($ToNumber(x));
_log1p_.summary = '返回 1 加上数值的自然对数';
_log1p_.params = { x: '要取对数的数' };
_log1p_.paramsType = { x: 'number' };
_log1p_.returnsType = 'number';

export const _log2_: VmLib = (x) => log2($ToNumber(x));
_log2_.summary = '返回数值的以 2 为底的对数';
_log2_.params = { x: '要取对数的数' };
_log2_.paramsType = { x: 'number' };
_log2_.returnsType = 'number';

export const _pow_: VmLib = (x, y) => pow($ToNumber(x), $ToNumber(y));
_pow_.summary = '返回 x 的 y 次幂';
_pow_.params = { x: '底数', y: '指数' };
_pow_.paramsType = { x: 'number', y: 'number' };
_pow_.returnsType = 'number';

export const _random_: VmLib = () => random();
_random_.summary = '返回 0 到 1 之间的伪随机数';
_random_.params = {};
_random_.paramsType = {};
_random_.returnsType = 'number';

export const _round_: VmLib = (x) => round($ToNumber(x));
_round_.summary = '返回四舍五入后的整数';
_round_.params = { x: '要四舍五入的数' };
_round_.paramsType = { x: 'number' };
_round_.returnsType = 'number';

export const _sign_: VmLib = (x) => sign($ToNumber(x));
_sign_.summary = '返回数值的符号（正数为 1，负数为 -1，零为 0）';
_sign_.params = { x: '要判断符号的数' };
_sign_.paramsType = { x: 'number' };
_sign_.returnsType = 'number';

export const _sin_: VmLib = (x) => sin($ToNumber(x));
_sin_.summary = '返回数值的正弦值';
_sin_.params = { x: '要计算正弦的数（弧度）' };
_sin_.paramsType = { x: 'number' };
_sin_.returnsType = 'number';

export const _sinh_: VmLib = (x) => sinh($ToNumber(x));
_sinh_.summary = '返回数值的双曲正弦值';
_sinh_.params = { x: '要计算双曲正弦的数' };
_sinh_.paramsType = { x: 'number' };
_sinh_.returnsType = 'number';

export const _sqrt_: VmLib = (x) => sqrt($ToNumber(x));
_sqrt_.summary = '返回数值的平方根';
_sqrt_.params = { x: '要开平方的数' };
_sqrt_.paramsType = { x: 'number' };
_sqrt_.returnsType = 'number';

export const _tan_: VmLib = (x) => tan($ToNumber(x));
_tan_.summary = '返回数值的正切值';
_tan_.params = { x: '要计算正切的数（弧度）' };
_tan_.paramsType = { x: 'number' };
_tan_.returnsType = 'number';

export const _tanh_: VmLib = (x) => tanh($ToNumber(x));
_tanh_.summary = '返回数值的双曲正切值';
_tanh_.params = { x: '要计算双曲正切的数' };
_tanh_.paramsType = { x: 'number' };
_tanh_.returnsType = 'number';

export const _trunc_: VmLib = (x) => trunc($ToNumber(x));
_trunc_.summary = '返回数值的整数部分（去除小数）';
_trunc_.params = { x: '要取整数部分的数' };
_trunc_.paramsType = { x: 'number' };
_trunc_.returnsType = 'number';

export { PI as '@pi', E as '@e' };

export const debug_print: VmLib = (...args) => {
    // eslint-disable-next-line no-console
    console.trace(...args);
};
debug_print.summary = '打印调试信息到控制台';
debug_print.params = {
    '..args': '要打印的调试信息，可以是任意类型',
};
debug_print.paramsType = {
    '..args': '[any]',
};
debug_print.returnsType = 'nil';
