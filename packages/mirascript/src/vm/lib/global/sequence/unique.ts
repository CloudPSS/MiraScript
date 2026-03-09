import { toBoolean } from '../../../../helpers/convert/to-boolean.js';
import { $Call } from '../../../operations/call.js';
import { isSame } from '../../../operations/utils.js';
import type { VmAny, VmConst, VmValue } from '../../../types/index.js';
import { expectArray, expectCallable, VmLib } from '../../helpers.js';

/** 默认相等 */
function defaultEqual(a: VmValue = null, b: VmValue = null): boolean {
    return isSame(a, b);
}

/** 获取相等函数 */
function eq(equaler: VmAny, recovered: VmValue): typeof defaultEqual {
    if (equaler == null) return defaultEqual;
    expectCallable('equal', equaler, recovered);
    return (a: VmValue = null, b: VmValue = null) => {
        const ret = $Call(equaler, [a, b]);
        return toBoolean(ret, undefined);
    };
}

export const unique = VmLib(
    (data, equal) => {
        expectArray('data', data, null);
        const e = eq(equal, data);
        const arr: VmConst[] = [];
        for (const v of data) {
            let found = false;
            for (const u of arr) {
                if (e(v, u)) {
                    found = true;
                    break;
                }
            }
            if (!found) arr.push(v ?? null);
        }
        return arr;
    },
    {
        summary: '对数组中的元素进行去重，并返回去重后的结果',
        params: {
            data: '要去重的数组',
            equal: '用于判等两个元素的函数，返回一个布尔值，默认使用严格相等比较',
        },
        paramsType: {
            data: 'array',
            equal: 'fn(a: any, b: any) -> boolean',
        },
        returnsType: 'array',
        examples: ['unique([1, 2, 2, 3]) // [1, 2, 3]'],
    },
);

export const unique_by = VmLib(
    (data, key, equal) => {
        expectArray('data', data, null);
        expectCallable('key', key, data);
        const e = eq(equal, data);
        const arr: VmConst[] = [];
        const keys: VmValue[] = [];
        const len = data.length;
        for (let i = 0; i < len; i++) {
            const v = data[i] ?? null;
            const k = $Call(key, [v, i, data]);
            let found = false;
            const keysLen = keys.length;
            for (let j = 0; j < keysLen; j++) {
                if (e(k, keys[j])) {
                    found = true;
                    break;
                }
            }
            if (!found) {
                arr.push(v);
                keys.push(k);
            }
        }
        return arr;
    },
    {
        summary: '根据键函数对数组中的元素进行去重，并返回去重后的结果',
        params: {
            data: '要去重的数组',
            key: '用于提取去重键的函数，接受一个元素并返回其去重键',
            equal: '用于判同两个元素的函数，返回一个布尔值，默认使用严格相等比较',
        },
        paramsType: {
            data: 'array',
            key: 'fn(value: any, index: number, arr: type(data)) -> any',
            equal: 'fn(a: any, b: any) -> boolean',
        },
        returnsType: 'array',
        examples: ['unique_by(["apple", "banana", "apricot"], fn { chars(it)[0] }) // ["apple", "banana"]'],
    },
);
