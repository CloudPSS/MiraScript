import type { VmArray, VmConst, VmRecord } from '../../../types/index.js';
import { toString } from '../../../../helpers/convert/to-string.js';
import { VM_ARRAY_MAX_LENGTH } from '../../../../helpers/constants.js';
import { keys as _keys, values as _values, entries as _entries, PositiveInfinity } from '../../../../helpers/utils.js';
import { $Call, $El, $Get } from '../../../operations/index.js';
import { VmLib, expectCallable, expectIntegerRange } from '../../helpers.js';

export const new_record = VmLib(
    (size, generator): VmRecord => {
        const n = expectIntegerRange('size', size, 0, PositiveInfinity);
        expectCallable('generator', generator, null);
        const result: Record<string, VmConst> = {};
        for (let i = 0; i < n; i++) {
            const entry = $Call(generator, [i]) as VmArray;
            if (entry == null) continue;
            const key = $Get(entry, 0);
            const value = $El($Get(entry, 1));
            result[toString(key)] = value;
        }
        return result;
    },
    {
        summary: '根据生成器函数创建一个记录',
        params: { size: '记录的大小', generator: '生成器函数，返回键值对或 nil（表示跳过该条目）' },
        paramsType: { size: 'number', generator: 'fn(index: number) -> (string, any) | nil' },
        returnsType: 'record',
        examples: ['new_record(3, fn { (it, it * 2) }) // (0, 2, 4)'],
    },
);

export const new_array = VmLib(
    (length, generator): VmArray => {
        const n = expectIntegerRange('length', length, 0, VM_ARRAY_MAX_LENGTH);
        expectCallable('generator', generator, null);
        const result: VmConst[] = [];
        for (let i = 0; i < n; i++) {
            result.push($El($Call(generator, [i])));
        }
        return result;
    },
    {
        summary: '根据生成器函数创建一个数组',
        params: { length: '数组的长度', generator: '生成器函数，返回数组元素的值' },
        paramsType: { length: 'number', generator: 'fn(index: number) -> any' },
        returnsType: 'array',
        examples: ['new_array(5, fn { it * it }) // [0, 1, 4, 9, 16]'],
    },
);
