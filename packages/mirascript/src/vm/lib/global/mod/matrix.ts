import { isVmArray, type VmConst } from '../../../types/index.js';
import { VmLib, expectArray } from '../../_helpers.js';

export const transpose = VmLib(
    (matrix) => {
        expectArray('matrix', matrix, matrix);
        if (matrix.length === 0) return [];
        const items = matrix.map((i) => (isVmArray(i) ? i : [i]));
        const numRows = matrix.length;
        const numCols = Math.max(...items.map((row) => row.length));
        const transposed: VmConst[][] = Array.from({ length: numCols }, () => Array.from({ length: numRows }));
        for (let i = 0; i < numRows; i++) {
            for (let j = 0; j < numCols; j++) {
                transposed[j]![i] = items[i]![j] ?? null;
            }
        }
        return transposed;
    },
    {
        summary: '转置矩阵',
        params: { matrix: '要转置的矩阵' },
        paramsType: { matrix: '[[any]]' },
        returnsType: '[[any]]',
    },
);
