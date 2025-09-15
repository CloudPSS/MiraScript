import { $Call, $ToNumber } from '../../../operations.js';
import type { VmAny, VmConst, VmValue } from '../../../types/index.js';
import { VmLib, expectArray, expectCallable } from '../../_helpers.js';

/** 默认比较 */
function defaultCompare(a: VmValue, b: VmValue): number {
    a ??= '';
    b ??= '';
    if (typeof a == 'string' && typeof b == 'string') {
        if (a < b) return -1;
        if (a > b) return 1;
        return 0;
    }
    if (Object.is(a, b)) return 0;
    const an = $ToNumber(a) || 0;
    const bn = $ToNumber(b) || 0;
    return an - bn;
}

/** 获取比较函数 */
function cmp(comparator: VmAny, recovered: VmValue): typeof defaultCompare {
    if (comparator == null) return defaultCompare;
    expectCallable('comparator', comparator, recovered);
    return (a: VmValue, b: VmValue) => {
        const ret = $Call(comparator, [a, b]);
        return $ToNumber(ret);
    };
}

export const sort = VmLib(
    (data, comparator) => {
        expectArray('data', data, null);
        const compare = cmp(comparator, data);
        const arr: VmConst[] = Array.from(data, (v) => v ?? null);
        arr.sort(compare);
        return arr;
    },
    {
        summary: '对数组中的元素进行排序，并返回排序后的结果',
        params: {
            data: '要排序的数组',
            comparator: '用于比较两个元素的函数，返回一个数字，表示它们的相对顺序，默认按升序排列',
        },
        paramsType: {
            data: 'array',
            comparator: 'fn(a: any, b: any) -> number',
        },
        returnsType: 'array',
    },
);

export const sort_by = VmLib(
    (data, key_fn, comparator) => {
        expectArray('data', data, null);
        expectCallable('key_fn', key_fn, data);
        const compare = cmp(comparator, data);
        const arr: Array<{ original: VmConst; key: VmValue }> = Array.from(data, (v, i) => ({
            original: v ?? null,
            key: $Call(key_fn, [v ?? null, i, data]),
        }));
        arr.sort((a, b) => compare(a.key, b.key));
        return arr.map((e) => e.original);
    },
    {
        summary: '根据键函数对数组中的元素进行排序，并返回排序后的结果',
        params: {
            data: '要排序的数组',
            key_fn: '用于提取排序键的函数，接受一个元素并返回其排序键',
            comparator: '用于比较两个排序键的函数，返回一个数字，表示它们的相对顺序，默认按升序排列',
        },
        paramsType: {
            data: 'array',
            key_fn: 'fn(value: any, index: number, arr: type(data)) -> any',
            comparator: 'fn(a: any, b: any) -> number',
        },
        returnsType: 'array',
    },
);
