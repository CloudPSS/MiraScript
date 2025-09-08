import { $Call, $ToBoolean } from '../../../operations.js';
import { isVmConst, type VmAny, type VmValue } from '../../../types/index.js';
import { VmLib, expectCallable, expectConst, map as mapImpl } from '../../_helpers.js';

export * from './with.js';
export * from './entries.js';
export * from './len.js';

/** map 和 filter 的实现 */
function mapImplWrapped(
    data: VmAny,
    fnName: string,
    fn: VmAny,
    mapper: (fn: VmValue, value: VmValue, index: number | string | null, data: VmValue) => VmValue | undefined,
): VmValue {
    expectConst('data', data, null);
    expectCallable(fnName, fn, data);
    return mapImpl(data, (value, index, data) => {
        const ret = mapper(fn, value, index, data);
        if (ret === undefined || isVmConst(ret)) return ret;
        return null;
    });
}

export const map = VmLib(
    (data, f) => mapImplWrapped(data, 'f', f, (fn, value, key, data) => $Call(fn, [value, key, data])),
    {
        summary: '对数组或记录中的每个元素应用函数，并返回结果',
        params: {
            data: '要映射的数组或记录',
            f: '应用于每个元素的函数',
        },
        paramsType: {
            data: 'array | record',
            f: 'fn(value: any, key: number | string | nil, input: type(data)) -> any',
        },
        returnsType: 'type(data)',
    },
);

export const filter = VmLib(
    (data, predicate) =>
        mapImplWrapped(data, 'predicate', predicate, (fn, value, key, data) => {
            const ret = $Call(fn, [value, key, data]);
            return $ToBoolean(ret) ? value : undefined;
        }),
    {
        summary: '过滤数组或记录中的元素，返回满足条件的元素',
        params: {
            data: '要过滤的数组或记录',
            predicate: '用于测试每个元素的函数，返回 true 或 false',
        },
        paramsType: {
            data: 'array | record',
            predicate: 'fn(value: any, key: number | string | nil, input: type(data)) -> boolean',
        },
        returnsType: 'type(data)',
    },
);

export const filter_map = VmLib(
    (data, f) =>
        mapImplWrapped(data, 'f', f, (fn, value, key, data) => {
            const ret = $Call(fn, [value, key, data]);
            return ret ?? undefined;
        }),
    {
        summary: '对数组或记录中的每个元素应用函数，并返回非 nil 的结果',
        params: {
            data: '要映射的数组或记录',
            f: '应用于每个元素的函数，返回 nil 或非 nil 的值',
        },
        paramsType: {
            data: 'array | record',
            f: 'fn(value: any, key: number | string | nil, input: type(data)) -> any | nil',
        },
        returnsType: 'type(data)',
    },
);
