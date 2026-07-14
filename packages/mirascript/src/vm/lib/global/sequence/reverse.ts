import { VmLib, expectArray } from '../../helpers.js';

export const reverse = VmLib(
    (arr) => {
        expectArray('arr', arr, null);
        const dup = [...arr];
        dup.reverse();
        return dup;
    },
    {
        summary: '返回数组的反转副本',
        params: { arr: { type: 'array', description: '要反转的数组' } },
        returns: { type: 'array' },
        examples: ['reverse([1, 2, 3]) // [3, 2, 1]'],
    },
);
