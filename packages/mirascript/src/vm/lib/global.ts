import { Cp, Element } from '../helpers.js';
import { $Call, $ToBoolean, $ToNumber, $ToString } from '../operations.js';
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
import { VmError } from '../error.js';
import {
    VmLib,
    expectArray,
    expectArrayOrRecord,
    expectCallable,
    expectCompound,
    required,
    rethrowError,
} from './helpers.js';

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

export const max = VmLib(
    (...args) => {
        const numbers = getMinMaxNumbers(args);
        return Math.max(...numbers);
    },
    {
        summary: '返回一组数中的最大值',
        params: { '..args': '要比较的数值' },
        paramsType: { '..args': '[number]' },
        returnsType: 'number',
    },
);

export const min = VmLib(
    (...args) => {
        const numbers = getMinMaxNumbers(args);
        return Math.min(...numbers);
    },
    {
        summary: '返回一组数中的最小值',
        params: { '..args': '要比较的数值' },
        paramsType: { '..args': '[number]' },
        returnsType: 'number',
    },
);

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

export const map = VmLib((data, f) => mapImpl(data, 'f', f, (fn, value, key, data) => $Call(fn, [value, key, data])), {
    summary: '对数组、记录或外部对象中的每个元素应用函数，并返回结果',
    params: {
        data: '要映射的数组、记录或外部对象',
        f: '应用于每个元素的函数',
    },
    paramsType: {
        data: 'array | record | extern',
        f: 'fn(value: any, key: number | string | nil, input: type(data)) -> any',
    },
    returnsType: 'type(data)',
});

export const filter = VmLib(
    (data, predicate) =>
        mapImpl(data, 'predicate', predicate, (fn, value, key, data) => {
            const ret = $Call(fn, [value, key, data]);
            return $ToBoolean(ret) ? value : undefined;
        }),
    {
        summary: '过滤数组、记录或外部对象中的元素，返回满足条件的元素',
        params: {
            data: '要过滤的数组、记录或外部对象',
            predicate: '用于测试每个元素的函数，返回 true 或 false',
        },
        paramsType: {
            data: 'array | record | extern',
            predicate: 'fn(value: any, key: number | string | nil, input: type(data)) -> boolean',
        },
        returnsType: 'type(data)',
    },
);

export const filter_map = VmLib(
    (data, f) =>
        mapImpl(data, 'f', f, (fn, value, key, data) => {
            const ret = $Call(fn, [value, key, data]);
            return ret ?? undefined;
        }),
    {
        summary: '对数组、记录或外部对象中的每个元素应用函数，并返回非 nil 的结果',
        params: {
            data: '要映射的数组、记录或外部对象',
            f: '应用于每个元素的函数，返回 nil 或非 nil 的值',
        },
        paramsType: {
            data: 'array | record | extern',
            f: 'fn(value: any, key: number | string | nil, input: type(data)) -> any | nil',
        },
        returnsType: 'type(data)',
    },
);

export const len = VmLib(
    (arr) => {
        expectArray(0, arr, Number.NaN);
        return arr.length;
    },
    {
        summary: '返回数组的长度',
        params: { arr: '要求长度的数组' },
        paramsType: { arr: 'array' },
        returnsType: 'number',
    },
);

export const chars = VmLib(
    (str) => {
        required('str', str, null);
        return [...$ToString(str)];
    },
    {
        summary: '将字符串转换为字符数组',
        params: { str: '要转换的字符串' },
        paramsType: { str: 'string' },
        returnsType: '[string]',
    },
);

export const to_json = VmLib(
    (data) => {
        if (isVmExtern(data)) {
            return JSON.stringify(data.value);
        }
        if (isVmModule(data)) {
            return '{}';
        }
        return JSON.stringify(data);
    },
    {
        summary: '将数据转换为 JSON 字符串',
        params: { data: '要转换为 JSON 的数据' },
        paramsType: { data: 'any' },
        returnsType: 'string',
    },
);

export const from_json = VmLib(
    (json, fallback) => {
        required('json', json, null);
        if (typeof json != 'string') return json;
        try {
            return JSON.parse(json);
        } catch (ex) {
            rethrowError('Invalid JSON', ex, fallback ?? null);
        }
    },
    {
        summary: '将 JSON 字符串转换为数据',
        params: { json: '要转换的 JSON 字符串', fallback: '如果转换失败，返回的默认值' },
        paramsType: { json: 'string', fallback: 'any' },
        returnsType: 'any',
    },
);

export const _with_ = VmLib(
    (data, ...entries) => {
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
    },
    {
        summary: '在数组或记录中设置多个键值对',
        params: { data: '要设置的数组或记录', '..entries': '要设置的键值对，成对出现' },
        paramsType: { data: 'array | record', '..entries': '[string | number | any]' },
        returnsType: 'type(data)',
    },
);

export const keys = VmLib(
    (data) => {
        expectCompound('data', data, []);
        if (isVmArray(data)) {
            return Array.from({ length: data.length }, (_, i) => $ToString(i));
        }
        if (isVmRecord(data)) {
            return Object.keys(data);
        }
        return data.keys();
    },
    {
        summary: '返回数组、记录、外部对象或模块的键列表',
        params: { data: '要获取键的数组、记录、外部对象或模块' },
        paramsType: { data: 'array | record | extern | module' },
        returnsType: '[string]',
    },
);

export const values = VmLib(
    (data) => {
        expectArrayOrRecord('data', data, []);
        if (isVmArray(data)) {
            return Array.from({ length: data.length }, (_, i) => data[i] ?? null);
        }
        return Object.values(data);
    },
    {
        summary: '返回数组或记录的值列表',
        params: { data: '要获取值的数组或记录' },
        paramsType: { data: 'array | record' },
        returnsType: 'array',
    },
);

export const entries = VmLib(
    (data) => {
        expectArrayOrRecord('data', data, []);
        if (isVmArray(data)) {
            return Array.from({ length: data.length }, (_, i) => [$ToString(i), data[i] ?? null]);
        }
        return Object.entries(data);
    },
    {
        summary: '返回数组或记录的键值对列表',
        params: { data: '要获取键值对的数组或记录' },
        paramsType: { data: 'array | record' },
        returnsType: '[(string, any)]',
    },
);

export const to_string = VmLib((data) => $ToString(data), {
    summary: '将数据转换为字符串',
    params: { data: '要转换的数据' },
    paramsType: { data: 'any' },
    returnsType: 'string',
});

export const to_number = VmLib((data) => $ToNumber(data), {
    summary: '将数据转换为数字',
    params: { data: '要转换的数据' },
    paramsType: { data: 'any' },
    returnsType: 'number',
});

export const to_boolean = VmLib((data) => $ToBoolean(data), {
    summary: '将数据转换为布尔值',
    params: { data: '要转换的数据' },
    paramsType: { data: 'any' },
    returnsType: 'boolean',
});

export const _abs_ = VmLib((x) => abs($ToNumber(x)), {
    summary: '返回数值的绝对值',
    params: { x: '要取绝对值的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _acos_ = VmLib((x) => acos($ToNumber(x)), {
    summary: '返回数值的反余弦值（弧度）',
    params: { x: '要计算反余弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _acosh_ = VmLib((x) => acosh($ToNumber(x)), {
    summary: '返回数值的反双曲余弦值',
    params: { x: '要计算反双曲余弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _asin_ = VmLib((x) => asin($ToNumber(x)), {
    summary: '返回数值的反正弦值（弧度）',
    params: { x: '要计算反正弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _asinh_ = VmLib((x) => asinh($ToNumber(x)), {
    summary: '返回数值的反双曲正弦值',
    params: { x: '要计算反双曲正弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _atan_ = VmLib((x) => atan($ToNumber(x)), {
    summary: '返回数值的反正切值（弧度）',
    params: { x: '要计算反正切的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _atan2_ = VmLib((x, y) => atan2($ToNumber(x), $ToNumber(y)), {
    summary: '返回从原点到点 (x, y) 的角度（弧度）',
    params: { x: 'x 坐标', y: 'y 坐标' },
    paramsType: { x: 'number', y: 'number' },
    returnsType: 'number',
});
export const _atanh_ = VmLib((x) => atanh($ToNumber(x)), {
    summary: '返回数值的反双曲正切值',
    params: { x: '要计算反双曲正切的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _cbrt_ = VmLib((x) => cbrt($ToNumber(x)), {
    summary: '返回数值的立方根',
    params: { x: '要计算立方根的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _ceil_ = VmLib((x) => ceil($ToNumber(x)), {
    summary: '返回大于等于给定数的最小整数',
    params: { x: '要向上取整的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _cos_ = VmLib((x) => cos($ToNumber(x)), {
    summary: '返回数值的余弦值',
    params: { x: '要计算余弦的数（弧度）' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _cosh_ = VmLib((x) => cosh($ToNumber(x)), {
    summary: '返回数值的双曲余弦值',
    params: { x: '要计算双曲余弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _exp_ = VmLib((x) => exp($ToNumber(x)), {
    summary: '返回 e 的指定次幂',
    params: { x: '指数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _expm1_ = VmLib((x) => expm1($ToNumber(x)), {
    summary: '返回 e 的 x 次幂减 1',
    params: { x: '指数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _floor_ = VmLib((x) => floor($ToNumber(x)), {
    summary: '返回小于等于给定数的最大整数',
    params: { x: '要向下取整的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _hypot_ = VmLib((...values) => hypot(...values.map($ToNumber)), {
    summary: '返回所有参数平方和的平方根',
    params: { '..values': '要计算的数值' },
    paramsType: { '..values': '[number]' },
    returnsType: 'number',
});
export const _log_ = VmLib((x) => log($ToNumber(x)), {
    summary: '返回数值的自然对数（以 e 为底）',
    params: { x: '要取对数的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _log10_ = VmLib((x) => log10($ToNumber(x)), {
    summary: '返回数值的以 10 为底的对数',
    params: { x: '要取对数的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _log1p_ = VmLib((x) => log1p($ToNumber(x)), {
    summary: '返回 1 加上数值的自然对数',
    params: { x: '要取对数的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _log2_ = VmLib((x) => log2($ToNumber(x)), {
    summary: '返回数值的以 2 为底的对数',
    params: { x: '要取对数的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _pow_ = VmLib((x, y) => pow($ToNumber(x), $ToNumber(y)), {
    summary: '返回 x 的 y 次幂',
    params: { x: '底数', y: '指数' },
    paramsType: { x: 'number', y: 'number' },
    returnsType: 'number',
});
export const _random_ = VmLib(() => random(), {
    summary: '返回 0 到 1 之间的伪随机数',
    params: {},
    paramsType: {},
    returnsType: 'number',
});
export const _round_ = VmLib((x) => round($ToNumber(x)), {
    summary: '返回四舍五入后的整数',
    params: { x: '要四舍五入的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _sign_ = VmLib((x) => sign($ToNumber(x)), {
    summary: '返回数值的符号（正数为 1，负数为 -1，零为 0）',
    params: { x: '要判断符号的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _sin_ = VmLib((x) => sin($ToNumber(x)), {
    summary: '返回数值的正弦值',
    params: { x: '要计算正弦的数（弧度）' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _sinh_ = VmLib((x) => sinh($ToNumber(x)), {
    summary: '返回数值的双曲正弦值',
    params: { x: '要计算双曲正弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _sqrt_ = VmLib((x) => sqrt($ToNumber(x)), {
    summary: '返回数值的平方根',
    params: { x: '要开平方的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _tan_ = VmLib((x) => tan($ToNumber(x)), {
    summary: '返回数值的正切值',
    params: { x: '要计算正切的数（弧度）' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _tanh_ = VmLib((x) => tanh($ToNumber(x)), {
    summary: '返回数值的双曲正切值',
    params: { x: '要计算双曲正切的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const _trunc_ = VmLib((x) => trunc($ToNumber(x)), {
    summary: '返回数值的整数部分（去除小数）',
    params: { x: '要取整数部分的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});

export { PI as '@pi', E as '@e' };

export const debug_print = VmLib(
    (...args) => {
        // eslint-disable-next-line no-console
        console.log('\u001B[46;30m MiraScript \u001B[0m', args);
    },
    {
        summary: '打印调试信息到控制台',
        params: { '..args': '要打印的调试信息，可以是任意类型' },
        paramsType: { '..args': '[any]' },
        returnsType: 'nil',
    },
);
