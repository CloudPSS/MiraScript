import { toBoolean } from '../../../../helpers/convert/to-boolean.js';
import { $Call } from '../../../operations/index.js';
import { isVmConst, type VmAny, type VmValue } from '../../../types/index.js';
import { VmLib, expectCallable, expectConst, map as mapImpl } from '../../helpers.js';

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
            data: { type: 'array | record', description: '要映射的数组或记录' },
            f: {
                type: 'fn(value: any, key: number | string, input: type(data)) -> any',
                description: '应用于每个元素的函数',
            },
        },
        returns: { type: 'type(data)' },
        examples: ['map([1, 2, 3], fn { it * it }) // [1, 4, 9]'],
    },
);

export const filter = VmLib(
    (data, predicate) =>
        mapImplWrapped(data, 'predicate', predicate, (fn, value, key, data) => {
            const ret = $Call(fn, [value, key, data]);
            return toBoolean(ret, undefined) ? value : undefined;
        }),
    {
        summary: '过滤数组或记录中的元素，返回满足条件的元素',
        params: {
            data: { type: 'array | record', description: '要过滤的数组或记录' },
            predicate: {
                type: 'fn(value: any, key: number | string, input: type(data)) -> boolean',
                description: '用于测试每个元素的函数',
            },
        },
        returns: { type: 'type(data)' },
        examples: ['filter([1, 2, 3, 4], fn { it % 2 == 0 }) // [2, 4]'],
    },
);

export const filter_map = VmLib(
    (data, f) =>
        mapImplWrapped(data, 'f', f, (fn, value, key, data) => {
            const ret = $Call(fn, [value, key, data]);
            return ret ?? undefined;
        }),
    {
        summary: '对数组或记录中的每个元素应用函数，并返回非 `nil` 的结果',
        params: {
            data: { type: 'array | record', description: '要映射的数组或记录' },
            f: {
                type: 'fn(value: any, key: number | string, input: type(data)) -> any | nil',
                description: '应用于每个元素的函数',
            },
        },
        returns: { type: 'type(data)' },
        examples: ['filter_map([1, 2, 3], fn {\n  if it % 2 == 0 { it * it } else { nil } \n}) // [4]'],
    },
);
