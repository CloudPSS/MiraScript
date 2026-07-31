import { toString } from '../../../../helpers/convert/index.js';
import { fromEntries } from '../../../../helpers/utils.js';
import { $Call } from '../../../operations/call.js';
import type { VmConst } from '../../../types/index.js';
import { VmLib, expectArray, expectCallable } from '../../helpers.js';

export const group_by = VmLib(
    (data, key) => {
        expectArray('data', data, null);
        expectCallable('key', key, data);
        const result = new Map<string, VmConst[]>();
        const len = data.length;
        for (let i = 0; i < len; i++) {
            const v = data[i] ?? null;
            const k = toString($Call(key, [v, i, data]), undefined);
            let arr = result.get(k);
            if (!arr) {
                arr = [];
                result.set(k, arr);
            }
            arr.push(v);
        }
        return fromEntries(result);
    },
    {
        summary: '根据指定的键函数对数组进行分组',
        params: {
            data: { type: 'array', description: '要分组的数组' },
            key: {
                type: 'fn(value: any, index: number, arr: type(data)) -> string',
                description: '用于生成分组键的函数',
            },
        },
        returns: { type: 'record' },
        examples: ['group_by([1, 2, 3, 4], fn { it % 2 }) // ("0": [2, 4], "1": [1, 3])'],
    },
);
