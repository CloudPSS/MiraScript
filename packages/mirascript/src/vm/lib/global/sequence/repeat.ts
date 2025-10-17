import { $ToNumber } from '../../../operations.ts';
import type { VmConst } from '../../../types/index.ts';
import { arrayLen, expectConst, required, VmLib } from '../../_helpers.ts';

export const repeat = VmLib(
    (data, times) => {
        expectConst('data', data, []);
        required('times', times, []);
        const n = arrayLen($ToNumber(times));
        const result: VmConst[] = [];
        result.length = n;
        result.fill(data);
        return result;
    },
    {
        summary: '创建一个包含重复元素的数组',
        params: {
            data: '要重复的元素',
            times: '重复的次数，必须是非负整数',
        },
        paramsType: {
            data: 'any',
            times: 'number',
        },
        returnsType: 'type(data)[]',
        examples: ['repeat(0, 5) // [0, 0, 0, 0, 0]', 'repeat("a", 3) // ["a", "a", "a"]'],
    },
);
