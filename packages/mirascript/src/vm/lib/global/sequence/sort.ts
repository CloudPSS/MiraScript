import { $Call, $ToNumber } from '../../../operations.js';
import type { VmAny, VmConst, VmValue } from '../../../types/index.js';
import { VmLib, expectArray, expectCallable } from '../../_helpers.js';

/** 默认比较 */
function defaultCompare(a: VmValue = null, b: VmValue = null): number {
    if (Object.is(a, b)) return 0;
    if ((typeof a == 'string' || a == null) && (typeof b == 'string' || b == null)) {
        a ??= '';
        b ??= '';
        if (a < b) return -1;
        if (a > b) return 1;
        return 0;
    }
    // nan is treated as 0
    const an = $ToNumber(a) || 0;
    const bn = $ToNumber(b) || 0;
    if (an < bn) return -1;
    if (an > bn) return 1;
    return 0;
}

/** 获取比较函数 */
function cmp(comparator: VmAny, recovered: VmValue): typeof defaultCompare {
    if (comparator == null) return defaultCompare;
    expectCallable('comparator', comparator, recovered);
    return (a: VmValue = null, b: VmValue = null) => {
        const ret = $Call(comparator, [a, b]);
        return $ToNumber(ret);
    };
}

export const sort = VmLib(
    (data, comparator) => {
        expectArray('data', data, null);
        const compare = cmp(comparator, data);
        const arr: VmConst[] = [];
        for (const v of data) {
            arr.push(v ?? null);
        }
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
        examples: ['sort(["c", "a", "b"]) // ["a", "b", "c"]'],
    },
);

export const sort_by = VmLib(
    (data, key_fn, comparator) => {
        expectArray('data', data, null);
        expectCallable('key_fn', key_fn, data);
        const compare = cmp(comparator, data);
        const arr: Array<{ o: VmConst; k: VmValue }> = [];
        const len = data.length;
        for (let i = 0; i < len; i++) {
            const v = data[i] ?? null;
            arr.push({
                o: v,
                k: $Call(key_fn, [v, i, data]),
            });
        }
        arr.sort((a, b) => compare(a.k, b.k));
        return arr.map((e) => e.o);
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
        examples: ['sort_by([(0, "x"), (2, "y"), (1, "z")], fn (item) { item[0] }) // [(0, "x"), (1, "z"), (2, "y")]'],
    },
);
