import { toNumber } from '../../../helpers/convert/to-number.js';
import { isArray } from '../../../helpers/utils.js';
import { Cp } from '../../helpers.js';
import { $Add, $Call, $Div, $Mul, $Sub } from '../../operations.js';
import { isVmArray, isVmConst, type VmAny, type VmArray, type VmConst, type VmValue } from '../../types/index.js';
import {
    VmLib,
    expectArray,
    expectCallable,
    expectConst,
    required,
    throwError,
    getNumbers,
    arrayLen,
    map,
    expectInteger,
} from '../helpers.js';

/** 计算尺寸 */
function sizeImpl(matrix: VmValue): [] | [number] | [number, number] {
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
function num(v: VmConst | undefined): number {
    return toNumber(v, undefined);
}

export const size = VmLib(
    (matrix) => {
        required('matrix', matrix, []);
        return sizeImpl(matrix);
    },
    {
        summary: '获取矩阵尺寸',
        params: { matrix: '要获取尺寸的矩阵' },
        paramsType: { matrix: 'any[][]' },
        returnsType: '[number, number]',
        examples: ['matrix.size([[1, 2], [3, 4]]) // [2, 2]'],
    },
);

export const transpose = VmLib(
    (matrix) => {
        required('matrix', matrix, []);
        const [numRows, numCols] = sizeImpl(matrix);
        if (numRows == null || numCols == null) return matrix; // 一维数组或空数组无需转置

        const transposed: VmConst[][] = [];
        for (let j = 0; j < numCols; j++) {
            Cp();
            const tj = [];
            for (let i = 0; i < numRows; i++) {
                const row = (matrix as VmConst[][])[i] ?? null;
                const item = row?.[j] ?? null;
                tj[i] = item;
            }
            transposed[j] = tj;
        }
        return transposed;
    },
    {
        summary: '转置矩阵',
        params: { matrix: '要转置的矩阵' },
        paramsType: { matrix: 'any[][]' },
        returnsType: 'any[][]',
        examples: ['matrix.transpose([[1, 2], [3, 4]]) // [[1, 3], [2, 4]]'],
    },
);

/** 逐项操作 */
function entrywiseImpl(
    a: VmConst,
    b: VmConst,
    f: (a: VmConst, b: VmConst) => VmConst,
    vvf?: (va: readonly VmConst[], vb: readonly VmConst[], ar: number, br: number) => VmConst,
    mmf?: (
        ma: readonly VmConst[][],
        mb: readonly VmConst[][],
        ar: number,
        ac: number,
        br: number,
        bc: number,
    ) => VmConst,
    vmf?: (va: readonly VmConst[], mb: readonly VmConst[][], al: number, br: number, bc: number) => VmConst,
    mvf?: (ma: readonly VmConst[][], vb: readonly VmConst[], ar: number, ac: number, bl: number) => VmConst,
): VmConst {
    let [ar, ac] = sizeImpl(a);
    let [br, bc] = sizeImpl(b);

    if (ar == null) {
        if (br == null) {
            // s/s
            return f(a, b);
        } else if (bc == null) {
            // s/v
            const result: VmConst[] = [];
            for (let r = 0; r < br; r++) {
                const bItem = (b as VmConst[])[r] ?? null;
                result[r] = f(a, bItem);
            }
            return result;
        } else {
            // s/m
            const result: VmConst[][] = [];
            for (let r = 0; r < br; r++) {
                const bRow = (b as VmConst[][])[r] ?? [];
                const rRow: VmConst[] = [];
                result[r] = rRow;
                for (let c = 0; c < bc; c++) {
                    const bItem = bRow[c] ?? null;
                    rRow[c] = f(a, bItem);
                }
            }
            return result;
        }
    }
    if (br == null) {
        if (ac == null) {
            // v/s
            const result: VmConst[] = [];
            for (let r = 0; r < ar; r++) {
                const aItem = (a as VmConst[])[r] ?? null;
                result[r] = f(aItem, b);
            }
            return result;
        } else {
            // m/s
            const result: VmConst[][] = [];
            for (let r = 0; r < ar; r++) {
                const aRow = (a as VmConst[][])[r] ?? [];
                const rRow: VmConst[] = [];
                result[r] = rRow;
                for (let c = 0; c < ac; c++) {
                    const aItem = aRow[c] ?? null;
                    rRow[c] = f(aItem, b);
                }
            }
            return result;
        }
    }
    if (ac == null && bc == null) {
        // v/v
        if (vvf != null) {
            return vvf(a as VmConst[], b as VmConst[], ar, br);
        }
        const rr = Math.max(ar, br);
        const result: VmConst[] = [];
        for (let r = 0; r < rr; r++) {
            const aItem = (a as VmConst[])[r] ?? null;
            const bItem = (b as VmConst[])[r] ?? null;
            result[r] = f(aItem, bItem);
        }
        return result;
    }

    // m/m (m/v v/m)
    if (ac == null) {
        // v/m
        if (vmf != null) {
            return vmf(a as VmConst[], b as VmConst[][], ar, br, bc!);
        }
        ac = ar!;
        ar = 1;
        a = [a];
    }
    if (bc == null) {
        // m/v
        if (mvf != null) {
            return mvf(a as VmConst[][], b as VmConst[], ar, ac, br);
        }
        bc = br!;
        br = 1;
        b = [b];
    }

    if (mmf != null) {
        return mmf(a as VmConst[][], b as VmConst[][], ar, ac, br, bc);
    }
    const rr = Math.max(ar, br);
    const rc = Math.max(ac, bc);
    const result: VmConst[][] = [];
    for (let r = 0; r < rr; r++) {
        const rRow: VmConst[] = [];
        result[r] = rRow;
        for (let c = 0; c < rc; c++) {
            const aItem = (a as VmConst[][])[ar === 1 ? 0 : r]?.[ac === 1 ? 0 : c] ?? null;
            const bItem = (b as VmConst[][])[br === 1 ? 0 : r]?.[bc === 1 ? 0 : c] ?? null;
            rRow[c] = f(aItem, bItem);
        }
    }
    return result;
}

export const entrywise = VmLib(
    (a, b, f) => {
        expectConst('a', a, null);
        expectConst('b', b, null);
        expectCallable('f', f, null);
        return entrywiseImpl(a, b, (a, b) => {
            Cp();
            const ret = $Call(f, [a, b]);
            if (!isVmConst(ret)) return null;
            return ret;
        });
    },
    {
        summary: '逐项操作',
        params: { a: '第一个操作数', b: '第二个操作数', f: '操作函数' },
        paramsType: { a: 'any | any[] | any[][]', b: 'any | any[] | any[][]', f: 'fn(a: any, b: any) -> any' },
        returnsType: 'any | any[] | any[][]',
        examples: [`matrix.entrywise([1, 2], [3, 4], fn (x, y) { x + y }) // [4, 6]`],
    },
);

export const add = VmLib(
    (a, b) => {
        expectConst('a', a, null);
        expectConst('b', b, null);
        return entrywiseImpl(a, b, $Add);
    },
    {
        summary: '逐项相加',
        params: { a: '第一个操作数', b: '第二个操作数' },
        paramsType: { a: 'number | number[] | number[][]', b: 'number | number[] | number[][]' },
        returnsType: 'number | number[] | number[][]',
        examples: ['matrix.add([1, 2], [3, 4]) // [4, 6]'],
    },
);

export const subtract = VmLib(
    (a, b) => {
        expectConst('a', a, null);
        expectConst('b', b, null);
        return entrywiseImpl(a, b, $Sub);
    },
    {
        summary: '逐项相减',
        params: { a: '第一个操作数', b: '第二个操作数' },
        paramsType: { a: 'number | number[] | number[][]', b: 'number | number[] | number[][]' },
        returnsType: 'number | number[] | number[][]',
        examples: ['matrix.subtract([3, 4], [1, 2]) // [2, 2]'],
    },
);

export const entrywise_multiply = VmLib(
    (a, b) => {
        expectConst('a', a, null);
        expectConst('b', b, null);
        return entrywiseImpl(a, b, $Mul);
    },
    {
        summary: '逐项相乘',
        params: { a: '第一个操作数', b: '第二个操作数' },
        paramsType: { a: 'number | number[] | number[][]', b: 'number | number[] | number[][]' },
        returnsType: 'number | number[] | number[][]',
        examples: ['matrix.entrywise_multiply([1, 2], [3, 4]) // [3, 8]'],
    },
);

export const entrywise_divide = VmLib(
    (a, b) => {
        expectConst('a', a, null);
        expectConst('b', b, null);
        return entrywiseImpl(a, b, $Div);
    },
    {
        summary: '逐项相除',
        params: { a: '第一个操作数', b: '第二个操作数' },
        paramsType: { a: 'number | number[] | number[][]', b: 'number | number[] | number[][]' },
        returnsType: 'number | number[] | number[][]',
        examples: ['matrix.entrywise_divide([4, 6], [2, 3]) // [2, 2]'],
    },
);

export const multiply = VmLib(
    (a, b) => {
        expectConst('a', a, null);
        expectConst('b', b, null);
        return entrywiseImpl(
            a,
            b,
            $Mul,
            (a, b, al, bl) => {
                const l = Math.max(al, bl);
                let s = 0;
                for (let i = 0; i < l; i++) {
                    s += num(a[i]) * num(b[i]);
                }
                return s;
            },
            (a, b, ar, ac, br, bc) => {
                if (ac !== br) throwError(`Incompatible matrix dimensions`, null);
                const result: VmConst[][] = [];
                for (let r = 0; r < ar; r++) {
                    const rRow: VmConst[] = [];
                    result[r] = rRow;
                    for (let c = 0; c < bc; c++) {
                        let item = 0;
                        for (let k = 0; k < ac; k++) {
                            item += num((a as VmConst[][])[r]?.[k]) * num((b as VmConst[][])[k]?.[c]);
                        }
                        rRow[c] = item;
                    }
                }
                return result;
            },
            (a, b, al, br, bc) => {
                if (al !== br) throwError(`Incompatible matrix dimensions`, null);
                const result: VmConst[] = [];
                for (let c = 0; c < bc; c++) {
                    let item = 0;
                    for (let k = 0; k < al; k++) {
                        item += num(a[k]) * num((b as VmConst[][])[k]?.[c]);
                    }
                    result[c] = item;
                }
                return result;
            },
            (a, b, ar, ac, bl) => {
                if (ac !== bl) throwError(`Incompatible matrix dimensions`, null);
                const result: VmConst[] = [];
                for (let r = 0; r < ar; r++) {
                    let item = 0;
                    for (let k = 0; k < ac; k++) {
                        item += num((a as VmConst[][])[r]?.[k]) * num(b[k]);
                    }
                    result[r] = item;
                }
                return result;
            },
        );
    },
    {
        summary: '矩阵相乘',
        params: { a: '第一个操作数', b: '第二个操作数' },
        paramsType: { a: 'number | number[] | number[][]', b: 'number | number[] | number[][]' },
        returnsType: 'number | number[] | number[][]',
        examples: ['matrix.multiply([[1, 2], [3, 4]], [5, 6]) // [17, 39]'],
    },
);

export const invert = VmLib(
    (a) => {
        expectConst('a', a, null);
        const [rows, cols] = sizeImpl(a);
        if (rows == null) return 1 / num(a); // 标量取倒数
        if (cols == null) return map(a, (v) => 1 / num(v)); // 向量按元素取倒数

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
        params: { a: '待求逆的矩阵' },
        paramsType: { a: 'number | number[][]' },
        returnsType: 'number | number[][]',
        examples: ['matrix.invert([[1, 2], [3, 4]]) // [[-2, 1], [1.5, -0.5]]'],
    },
);

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
    params: { '..size': '矩阵的维度' },
    paramsType: { '..size': 'number[]' },
    returnsType: 'number[][]',
    examples: ['matrix.zeros(2, 3) // [[0, 0, 0], [0, 0, 0]]'],
});

export const ones = VmLib((...size) => filled(size, 1), {
    summary: '创建一个全一的矩阵',
    params: { '..size': '矩阵的维度' },
    paramsType: { '..size': 'number[]' },
    returnsType: 'number[][]',
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
        params: { '..size': '矩阵的维度' },
        paramsType: { '..size': '[number] | [number, number]' },
        returnsType: 'number[][]',
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
        params: { x: '对角线元素或要获取对角线的矩阵', k: '对角线偏移量，默认为 0' },
        paramsType: { x: 'number[] | number[][]', k: 'number' },
        returnsType: 'number[][] | number[]',
        examples: [
            'matrix.diagonal([1, 2, 3]) // [[1, 0, 0], [0, 2, 0], [0, 0, 3]]',
            'matrix.diagonal([[1, 2], [3, 4]]) // [1, 4]',
        ],
    },
);
