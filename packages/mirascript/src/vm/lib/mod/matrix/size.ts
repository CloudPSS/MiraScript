import { VmLib, required } from '../../helpers.js';
import { size as sizeImpl } from './helpers.js';

export const size = VmLib(
    (matrix) => {
        required('matrix', matrix, []);
        return sizeImpl(matrix);
    },
    {
        summary: '获取矩阵尺寸',
        params: { matrix: { type: 'any[][]', description: '要获取尺寸的矩阵' } },
        returns: { type: '[number, number]' },
        examples: ['matrix.size([[1, 2], [3, 4]]) // [2, 2]'],
    },
);
