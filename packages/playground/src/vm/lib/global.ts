import { Cp } from '../helpers';
import { $CallDyn, $ToBool, $ToNumber, $ToString } from '../operations';
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
import type { VmFunctionLike } from '../types/function.js';
import { VmError } from '../error';
import { arrayRequired, required, rethrowError } from './helpers';

/** Get the minimum and maximum numbers from the arguments. */
function getMinMaxNumbers(args: readonly VmAny[]): number[] {
    if (args.length === 0) return [];
    if (isVmArray(args[0])) args = args[0];
    const numbers: number[] = [];
    for (const arg of args) {
        if (arg === undefined) continue;
        numbers.push($ToNumber(arg));
    }
    return numbers;
}

export const max: VmFunctionLike = (...args) => {
    const numbers = getMinMaxNumbers(args);
    return Math.max(...numbers);
};
export const min: VmFunctionLike = (...args) => {
    const numbers = getMinMaxNumbers(args);
    return Math.min(...numbers);
};

/** map 和 filter 的实现 */
function mapImpl(
    data: VmAny,
    fn: VmAny,
    mapper: (fn: VmValue, value: VmValue, index: number | string | null, data: VmValue) => VmValue | undefined,
): VmValue {
    required('data', data, null);
    required('fn', fn, data);
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

export const map: VmFunctionLike = (data, fn) => {
    return mapImpl(data, fn, (fn, value, key, data) => {
        return $CallDyn(fn, [value, key, data]);
    });
};

export const filter: VmFunctionLike = (data, fn) => {
    return mapImpl(data, fn, (fn, value, key, data) => {
        const ret = $CallDyn(fn, [value, key, data]);
        return $ToBool(ret) ? value : undefined;
    });
};

export const filter_map: VmFunctionLike = (data, fn) => {
    return mapImpl(data, fn, (fn, value, key, data) => {
        const ret = $CallDyn(fn, [value, key, data]);
        return ret ?? undefined;
    });
};

export const len: VmFunctionLike = (arr) => {
    arrayRequired(0, arr, Number.NaN);
    return arr.length;
};

export const to_json: VmFunctionLike = (data) => {
    if (isVmExtern(data)) {
        return JSON.stringify(data.value);
    }
    if (isVmModule(data)) {
        return '{}';
    }
    return JSON.stringify(data);
};

export const from_json: VmFunctionLike = (json, fallback) => {
    required('json', json, null);
    if (typeof json != 'string') return json;
    try {
        return JSON.parse($ToString(json));
    } catch (ex) {
        rethrowError('Invalid JSON', ex, fallback ?? null);
    }
};
