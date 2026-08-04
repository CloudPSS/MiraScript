import { toNumber } from '../../../../helpers/convert/index.js';
import { isVmArray, type VmConst, type VmValue } from '../../../types/index.js';

/** 计算尺寸 */
export function size(matrix: VmValue): [] | [number] | [number, number] {
    if (!isVmArray(matrix)) return [];
    if (matrix.length === 0) return [0];

    const numRows = matrix.length;
    let numCols = 0;

    for (const row of matrix) {
        if (isVmArray(row)) {
            numCols = Math.max(numCols, row.length);
        } else {
            return [numRows];
        }
    }

    return [numRows, numCols];
}

/** 数组元素转为 number */
export function num(v: VmConst | undefined): number {
    return toNumber(v, undefined);
}
