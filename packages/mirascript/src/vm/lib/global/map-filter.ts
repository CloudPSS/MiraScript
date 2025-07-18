import { Cp } from '../../helpers.js';
import { $Call, $ToBoolean } from '../../operations.js';
import {
    isVmArray,
    isVmConst,
    isVmExtern,
    isVmPrimitive,
    isVmRecord,
    VmExtern,
    type VmAny,
    type VmArray,
    type VmConst,
    type VmRecord,
    type VmValue,
} from '../../types/index.js';
import { VmError } from '../../error.js';
import { VmLib, expectCallable, required } from '../helpers.js';

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
