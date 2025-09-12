import { VmLib, expectArray } from '../../_helpers.js';

export const reverse = VmLib(
    (arr) => {
        expectArray('arr', arr, null);
        const dup = [...arr];
        dup.reverse();
        return dup;
    },
    {
        summary: '返回数组的反转副本',
        params: { arr: '要反转的数组' },
        paramsType: { arr: 'array' },
        returnsType: 'array',
    },
);
