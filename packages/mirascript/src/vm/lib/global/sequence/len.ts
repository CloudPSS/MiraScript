import { NotNumber } from '../../../../helpers/utils.js';
import { isVmExtern } from '../../../types/index.js';
import { VmLib, expectArray, throwUnexpectedTypeError } from '../../helpers.js';

export const len = VmLib(
    (arr) => {
        if (isVmExtern(arr)) {
            if (!arr.isArrayLike()) {
                throwUnexpectedTypeError('arr', 'array-like extern', arr, NotNumber);
            }
            return arr.value.length;
        }
        expectArray('arr', arr, NotNumber);
        return arr.length;
    },
    {
        summary: '返回数组的长度',
        params: { arr: '要求长度的数组' },
        paramsType: { arr: 'array | extern' },
        returnsType: 'number',
        examples: ['len([1, 2, 3]) // 3'],
    },
);
