import { Cp } from '../../../checkpoint.js';
import { $Add, $Call, $Div, $Mul, $Sub } from '../../../operations/index.js';
import { isVmConst, type VmConst } from '../../../types/index.js';
import { VmLib, expectCallable, expectConst, throwError } from '../../helpers.js';
import { num, size } from './helpers.js';
const { max } = Math;

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
    let [ar, ac] = size(a);
    let [br, bc] = size(b);

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
        const rr = max(ar, br);
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
    const rr = max(ar, br);
    const rc = max(ac, bc);
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
        params: {
            a: { type: 'any | any[] | any[][]', description: '第一个操作数' },
            b: { type: 'any | any[] | any[][]', description: '第二个操作数' },
            f: { type: 'fn(a: any, b: any) -> any', description: '操作函数' },
        },
        returns: { type: 'any | any[] | any[][]' },
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
        params: {
            a: { type: 'number | number[] | number[][]', description: '第一个操作数' },
            b: { type: 'number | number[] | number[][]', description: '第二个操作数' },
        },
        returns: { type: 'number | number[] | number[][]' },
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
        params: {
            a: { type: 'number | number[] | number[][]', description: '第一个操作数' },
            b: { type: 'number | number[] | number[][]', description: '第二个操作数' },
        },
        returns: { type: 'number | number[] | number[][]' },
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
        params: {
            a: { type: 'number | number[] | number[][]', description: '第一个操作数' },
            b: { type: 'number | number[] | number[][]', description: '第二个操作数' },
        },
        returns: { type: 'number | number[] | number[][]' },
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
        params: {
            a: { type: 'number | number[] | number[][]', description: '第一个操作数' },
            b: { type: 'number | number[] | number[][]', description: '第二个操作数' },
        },
        returns: { type: 'number | number[] | number[][]' },
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
                const l = max(al, bl);
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
        params: {
            a: { type: 'number | number[] | number[][]', description: '第一个操作数' },
            b: { type: 'number | number[] | number[][]', description: '第二个操作数' },
        },
        returns: { type: 'number | number[] | number[][]' },
        examples: ['matrix.multiply([[1, 2], [3, 4]], [5, 6]) // [17, 39]'],
    },
);
