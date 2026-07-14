import type { VmConst } from '../../../types/index.js';
import { arrayLen, expectConst, expectNumber, VmLib } from '../../helpers.js';

export const repeat = VmLib(
    (data, times) => {
        expectConst('data', data, []);
        const n = arrayLen(expectNumber('times', times));
        const result: VmConst[] = [];
        result.length = n;
        result.fill(data);
        return result;
    },
    {
        summary: '创建一个包含重复元素的数组',
        params: {
            data: { type: 'any', description: '要重复的元素' },
            times: { type: 'number', description: '重复的次数，必须是非负整数' },
        },
        returns: { type: 'type(data)[]' },
        examples: ['repeat(0, 5) // [0, 0, 0, 0, 0]', 'repeat("a", 3) // ["a", "a", "a"]'],
    },
);
