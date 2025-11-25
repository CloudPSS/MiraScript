import { VmLib, expectArray, expectNumber } from '../../helpers.js';

export const flatten = VmLib(
    (data, depth = 1) => {
        expectArray('data', data, data);
        return data.flat(expectNumber('depth', depth) as 1);
    },
    {
        summary: '将数组扁平化',
        params: { data: '要扁平化的数组', depth: '扁平化的深度，默认为 1' },
        paramsType: { data: 'array', depth: 'number' },
        returnsType: 'array',
        examples: ['flatten([[1, 2], [3, [4]]], 2) // [1, 2, 3, 4]'],
    },
);
