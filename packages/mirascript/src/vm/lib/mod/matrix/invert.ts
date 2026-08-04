import type { VmConst } from '../../../types/index.js';
import { size, num } from './helpers.js';
import { VmLib, expectConst, throwError, iterate } from '../../helpers.js';

export const invert = VmLib(
    (a) => {
        expectConst('a', a, null);
        const [rows, cols] = size(a);
        if (rows == null) return 1 / num(a); // 标量取倒数
        if (cols == null) return iterate(a, (v) => 1 / num(v)); // 向量按元素取倒数

        if (rows !== cols) throwError(`Matrix must be square`, a);
        const m = a as VmConst[][];
        // https://github.com/josdejong/mathjs
        if (rows === 1) {
            // 1x1 矩阵
            const e = num(m[0]?.[0]);
            // if (e === 0) {
            //     throwError(`Matrix is singular`, null);
            // }
            return [[1 / e]];
        }
        if (rows === 2) {
            // 2x2 矩阵
            const a = num(m[0]?.[0]);
            const b = num(m[0]?.[1]);
            const c = num(m[1]?.[0]);
            const d = num(m[1]?.[1]);

            const det = a * d - b * c;
            // if (det === 0) throwError(`Matrix is singular`, null);
            return [
                [d / det, -b / det],
                [-c / det, a / det],
            ];
        }

        // 更高阶矩阵 使用高斯消元法

        // 初始化输入
        const A: number[][] = [];
        // 初始化结果为单位矩阵
        const B: number[][] = [];
        for (let r = 0; r < rows; r++) {
            const Ar: number[] = [];
            const Br: number[] = [];
            A[r] = Ar;
            B[r] = Br;
            for (let c = 0; c < cols; c++) {
                Ar[c] = num(m[r]?.[c]);
                Br[c] = r === c ? 1 : 0;
            }
        }

        // loop over all columns, and perform row reductions
        for (let c = 0; c < cols; c++) {
            // Pivoting: Swap row c with row r, where row r contains the largest element A[r][c]
            let ABig = Math.abs(A[c]![c]!);
            let rBig = c;
            let r = c + 1;
            while (r < rows) {
                if (Math.abs(A[r]![c]!) > ABig) {
                    ABig = Math.abs(A[r]![c]!);
                    rBig = r;
                }
                r++;
            }
            // if (ABig === 0) {
            //     throwError(`Matrix is singular`, null);
            // }
            r = rBig;
            if (r !== c) {
                const temp1 = A[c]!;
                A[c] = A[r]!;
                A[r] = temp1;
                const temp2 = B[c]!;
                B[c] = B[r]!;
                B[r] = temp2;
            }

            // eliminate non-zero values on the other rows at column c
            const Ac = A[c]!;
            const Bc = B[c]!;
            for (r = 0; r < rows; r++) {
                const Ar = A[r]!;
                const Br = B[r]!;
                if (r !== c) {
                    // eliminate value at column c and row r
                    if (Ar[c] !== 0) {
                        const f = -Ar[c]! / Ac[c]!;

                        // add (f * row c) to row r to eliminate the value
                        // at column c
                        for (let s = c; s < cols; s++) {
                            Ar[s] = Ar[s]! + f * Ac[s]!;
                        }
                        for (let s = 0; s < cols; s++) {
                            Br[s] = Br[s]! + f * Bc[s]!;
                        }
                    }
                } else {
                    // normalize value at Acc to 1,
                    // divide each value on row r with the value at Acc
                    const f = Ac[c]!;
                    for (let s = c; s < cols; s++) {
                        Ar[s] = Ar[s]! / f;
                    }
                    for (let s = 0; s < cols; s++) {
                        Br[s] = Br[s]! / f;
                    }
                }
            }
        }
        return B;
    },
    {
        summary: '矩阵求逆',
        params: { a: { type: 'number | number[][]', description: '待求逆的矩阵' } },
        returns: { type: 'number | number[][]' },
        examples: ['matrix.invert([[1, 2], [3, 4]]) // [[-2, 1], [1.5, -0.5]]'],
    },
);
