import { NotNumber } from '../../../../helpers/utils.js';
import { VmLib, expectArray } from '../../helpers.js';

export const len = VmLib(
    (arr) => {
        expectArray('arr', arr, NotNumber);
        return arr.length;
    },
    {
        summary: '返回数组的长度',
        params: { arr: '要求长度的数组' },
        paramsType: { arr: 'array' },
        returnsType: 'number',
        examples: ['len([1, 2, 3]) // 3'],
    },
);
