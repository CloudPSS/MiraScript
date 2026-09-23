import { isNaN, PositiveInfinity } from '../../../../helpers/utils.js';
import { isVmRecord, type VmAny } from '../../../types/index.js';
import { expectNumber, VmLib } from '../../helpers.js';
const { abs } = Math;
const { is } = Object;

/** Internal Cartesian pairs. Public values are records, never arrays. */
export type C = readonly [number, number];
/** Public complex number record type. */
export type Complex = { readonly 0: number; readonly 1: number };

export const inf = PositiveInfinity;
export const copySign = (x: number, y: number): number => (y < 0 || is(y, -0) ? -abs(x) : abs(x));
// A vanishing Cartesian factor remains zero even when the radial factor overflows.
export const radial = (r: number, x: number): number => (x === 0 && !isNaN(r) ? copySign(0, r) * x : r * x);

/** Parse a scalar or a two-field record using the ordinary numeric conversion. */
function parse(value: VmAny, name: string): C {
    if (isVmRecord(value) && '0' in value && '1' in value) {
        return [expectNumber(`${name}.0`, value['0']), expectNumber(`${name}.1`, value['1'])];
    }
    return [expectNumber(name, value), 0];
}
export const record = ([re, im]: C): Complex => ({ 0: re, 1: im });
/** Wrap a unary kernel with record conversion and help metadata. */
export function unary(f: (z: C) => C, summary: string): ((z: VmAny) => Complex) & VmLib {
    return VmLib((z: VmAny) => record(f(parse(z, 'z'))), {
        summary,
        params: { z: { type: 'number | (number, number)', description: '复数 record 或可转换为数字的值' } },
        returns: { type: '(number, number)' },
    });
}
/** Wrap a binary kernel with record conversion and help metadata. */
export function binary(f: (a: C, b: C) => C, summary: string): ((a: VmAny, b: VmAny) => Complex) & VmLib {
    return VmLib((a: VmAny, b: VmAny) => record(f(parse(a, 'a'), parse(b, 'b'))), {
        summary,
        params: {
            a: { type: 'number | (number, number)', description: '左操作数' },
            b: { type: 'number | (number, number)', description: '右操作数' },
        },
        returns: { type: '(number, number)' },
    });
}
/** Wrap a complex-input function whose result is always real. */
export function scalar(f: (z: C) => number, summary: string): ((z: VmAny) => number) & VmLib {
    return VmLib((z: VmAny) => f(parse(z, 'z')), {
        summary,
        params: { z: { type: 'number | (number, number)', description: '复数 record 或可转换为数字的值' } },
        returns: { type: 'number' },
    });
}
