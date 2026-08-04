import { isArray } from '../../../../helpers/utils.js';
import { Cp } from '../../../checkpoint.js';
import type { VmAny, VmArray, VmConst } from '../../../types/index.js';
import { VmLib, expectArray, throwError, getNumbers, arrayLen, expectInteger } from '../../helpers.js';

/** 填充 */
function filled(size: readonly VmAny[], value: VmConst): VmArray {
    const s = getNumbers(size);
    if (s.length === 0) return [];
    while (s.length > 0) {
        const repeat = arrayLen(s.pop());
        Cp();
        const data: VmConst[] = [];
        data.length = repeat;
        // 从 MiraScript 语义而言，可以使用同一个引用
        data.fill(value);
        value = data;
    }
    return value as VmArray;
}

export const zeros = VmLib((...size) => filled(size, 0), {
    summary: '创建一个全零的矩阵',
    params: { '..size': { type: 'number[]', description: '矩阵的维度' } },
    returns: { type: 'number[][]' },
    examples: ['matrix.zeros(2, 3) // [[0, 0, 0], [0, 0, 0]]'],
});

export const ones = VmLib((...size) => filled(size, 1), {
    summary: '创建一个全一的矩阵',
    params: { '..size': { type: 'number[]', description: '矩阵的维度' } },
    returns: { type: 'number[][]' },
    examples: ['matrix.ones(2, 2) // [[1, 1], [1, 1]]'],
});

export const identity = VmLib(
    (...size) => {
        let s = getNumbers(size);
        if (s.length === 0) return [];
        if (s.length > 2) throwError('Invalid matrix size', []);
        if (s.length === 1) s = [s[0]!, s[0]!];
        const m = arrayLen(s[0]);
        const n = arrayLen(s[1]);
        // 由于 `filled` 函数返回只读数组，其每行为相同引用，这里需要手动创建每行
        const ret: number[][] = [];
        for (let i = 0; i < m; i++) {
            const row: number[] = [];
            ret[i] = row;
            row.length = n;
            row.fill(0);
            if (i < n) row[i] = 1;
        }
        return ret;
    },
    {
        summary: '创建一个单位矩阵',
        params: { '..size': { type: '[number] | [number, number]', description: '矩阵的维度' } },
        returns: { type: 'number[][]' },
        examples: ['matrix.identity(3) // [[1, 0, 0], [0, 1, 0], [0, 0, 1]]'],
    },
);

export const diagonal = VmLib(
    (x, k = 0) => {
        expectArray('x', x, []);
        const fk = expectInteger('k', k);
        if (x.every((e) => isArray(e))) {
            // 获取对角线元素
            const diag: VmConst[] = [];
            for (let i = 0; i < x.length; i++) {
                const row = x[i] as VmArray | undefined;
                const r = i + fk;
                if (r < 0) continue;
                if (!row || r >= row.length) break;
                diag.push(row[r] ?? null);
            }
            return diag;
        }
        // 创建对角矩阵
        const l = x.length;
        const m = arrayLen(fk < 0 ? l - fk : l);
        const n = arrayLen(fk > 0 ? l + fk : l);
        const result: VmConst[][] = [];
        for (let i = 0; i < m; i++) {
            const row: VmConst[] = [];
            result[i] = row;
            row.length = n;
            row.fill(0);
            for (let j = 0; j < n; j++) {
                if (i + fk === j) {
                    row[j] = x[fk >= 0 ? i : j] ?? null;
                }
            }
        }
        return result;
    },
    {
        summary: '创建一个对角矩阵或获取矩阵的对角线',
        params: {
            x: { type: 'number[] | number[][]', description: '对角线元素或要获取对角线的矩阵' },
            k: { type: 'number', description: '对角线偏移量，默认为 0' },
        },
        returns: { type: 'number[][] | number[]' },
        examples: [
            'matrix.diagonal([1, 2, 3]) // [[1, 0, 0], [0, 2, 0], [0, 0, 3]]',
            'matrix.diagonal([[1, 2], [3, 4]]) // [1, 4]',
        ],
    },
);
