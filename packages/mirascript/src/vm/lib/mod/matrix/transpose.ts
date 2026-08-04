import { Cp } from '../../../checkpoint.js';
import type { VmConst } from '../../../types/index.js';
import { VmLib, required } from '../../helpers.js';
import { size } from './helpers.js';

export const transpose = VmLib(
    (matrix) => {
        required('matrix', matrix, []);
        const [numRows, numCols] = size(matrix);
        if (numRows == null || numCols == null) return matrix; // 一维数组或空数组无需转置

        const m = matrix as VmConst[][];
        const transposed: VmConst[][] = [];
        for (let j = 0; j < numCols; j++) {
            Cp();
            const tj = [];
            for (let i = 0; i < numRows; i++) {
                const row = m[i] ?? null;
                const item = row?.[j] ?? null;
                tj[i] = item;
            }
            transposed[j] = tj;
        }
        return transposed;
    },
    {
        summary: '转置矩阵',
        params: { matrix: { type: 'any[][]', description: '要转置的矩阵' } },
        returns: { type: 'any[][]' },
        examples: ['matrix.transpose([[1, 2], [3, 4]]) // [[1, 3], [2, 4]]'],
    },
);
