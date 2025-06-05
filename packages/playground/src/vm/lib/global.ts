import { Cp, Element } from '../helpers';
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
import { VmError } from '../error';
import { expectArray, expectArrayOrRecord, expectCallable, required, rethrowError } from './helpers';
import type { VmLib } from './loader';

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
export const min: VmLib = (...args) => {
    const numbers = getMinMaxNumbers(args);
    return Math.min(...numbers);
};

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

export const filter: VmLib = (data, predicate) => {
    return mapImpl(data, 'predicate', predicate, (fn, value, key, data) => {
        const ret = $CallDyn(fn, [value, key, data]);
        return $ToBool(ret) ? value : undefined;
    });
};

export const filter_map: VmLib = (data, f) => {
    return mapImpl(data, 'f', f, (fn, value, key, data) => {
        const ret = $CallDyn(fn, [value, key, data]);
        return ret ?? undefined;
    });
};

export const len: VmLib = (arr) => {
    expectArray(0, arr, Number.NaN);
    return arr.length;
};

export const chars: VmLib = (str) => {
    required('str', str, null);
    return [...$ToString(str)];
};

export const to_json: VmLib = (data) => {
    if (isVmExtern(data)) {
        return JSON.stringify(data.value);
    }
    if (isVmModule(data)) {
        return '{}';
    }
    return JSON.stringify(data);
};

export const from_json: VmLib = (json, fallback) => {
    required('json', json, null);
    if (typeof json != 'string') return json;
    try {
        return JSON.parse($ToString(json));
    } catch (ex) {
        rethrowError('Invalid JSON', ex, fallback ?? null);
    }
};

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

const { PI, E } = Math;
export { PI as '@pi', E as '@e' };
