import { isVmRecord, type VmAny } from '../../../types/index.js';
import { expectNumber, VmLib } from '../../helpers.js';
import * as k from './kernel.js';

/** Parse a scalar or a two-field record using the ordinary numeric conversion. */
function parse(value: VmAny, name: string): k.C {
    if (isVmRecord(value) && '0' in value && '1' in value) {
        return [expectNumber(`${name}.0`, value['0']), expectNumber(`${name}.1`, value['1'])];
    }
    return [expectNumber(name, value), 0];
}
const record = ([re, im]: k.C) => ({ 0: re, 1: im });
/** Wrap a unary kernel with record conversion and help metadata. */
function unary(f: (z: k.C) => k.C, summary: string) {
    return VmLib((z: VmAny) => record(f(parse(z, 'z'))), {
        summary,
        params: { z: { type: 'number | (number, number)', description: '复数 record 或可转换为数字的值' } },
        returns: { type: '(number, number)' },
    });
}
/** Wrap a binary kernel with record conversion and help metadata. */
function binary(f: (a: k.C, b: k.C) => k.C, summary: string) {
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
function scalar(f: (z: k.C) => number, summary: string) {
    return VmLib((z: VmAny) => f(parse(z, 'z')), {
        summary,
        params: { z: { type: 'number | (number, number)', description: '复数 record 或可转换为数字的值' } },
        returns: { type: 'number' },
    });
}
export const I = VmLib(record([0, 1]), { summary: '虚数单位 (0, 1)' });
export const from = unary((z) => z, '按数字转换语义将输入转换为 (实部, 虚部)，忽略额外字段');
export const real = scalar((z) => z[0], '返回复数的实部');
export const imag = scalar((z) => z[1], '返回复数的虚部');
export const abs = scalar((z) => Math.hypot(...z), '返回复数的模');
export const arg = scalar((z) => Math.atan2(z[1], z[0]), '返回复数的主辐角（弧度，保留符号零）');
export const conj = unary(k.conj, '返回共轭复数');
export const neg = unary(k.neg, '返回复数的相反数');
export const add = binary(k.add, '返回两个复数的和');
export const subtract = binary(k.subtract, '返回两个复数的差');
export const multiply = binary(k.multiply, '返回两个复数的积');
export const divide = binary(k.divide, '返回两个复数的商');
export const pow = binary(k.power, '返回复数幂的主值');
export const sqrt = unary(k.squareRoot, '返回复数平方根的主值');
export const cbrt = unary(k.cubeRoot, '返回复数立方根的主值');
export const exp = unary(k.exponential, '返回复数指数');
export const expm1 = unary(k.exponentialMinusOne, '返回复数指数减一，保留近零精度');
export const log = unary(k.logarithm, '返回复数自然对数的主值');
export const log1p = unary(k.logarithmOnePlus, '返回一加复数的自然对数主值，保留近零精度');
export const log2 = unary((z) => k.logBase(z, Math.LN2), '返回以 2 为底的复数对数主值');
export const log10 = unary((z) => k.logBase(z, Math.LN10), '返回以 10 为底的复数对数主值');
export const sin = unary(k.sine, '返回复数正弦');
export const cos = unary(k.cosine, '返回复数余弦');
export const tan = unary(k.tangent, '返回复数正切');
export const sinh = unary(k.hyperbolicSine, '返回复数双曲正弦');
export const cosh = unary(k.hyperbolicCosine, '返回复数双曲余弦');
export const tanh = unary(k.hyperbolicTangent, '返回复数双曲正切');
export const asin = unary(k.arcSine, '返回复数反正弦主值');
export const acos = unary(k.arcCosine, '返回复数反余弦主值');
export const atan = unary(k.arcTangent, '返回复数反正切主值');
export const asinh = unary(k.areaHyperbolicSine, '返回复数反双曲正弦主值');
export const acosh = unary(k.areaHyperbolicCosine, '返回复数反双曲余弦主值');
export const atanh = unary(k.areaHyperbolicTangent, '返回复数反双曲正切主值');
export const polar = VmLib(
    (r: VmAny, theta: VmAny) => record(k.polar(expectNumber('r', r), expectNumber('theta', theta))),
    {
        summary: '从模和辐角（弧度）构造复数',
        params: {
            r: { type: 'number', description: '径向长度' },
            theta: { type: 'number', description: '辐角（弧度）' },
        },
        returns: { type: '(number, number)' },
        examples: ['complex.polar(2, 0) // (2, 0)'],
    },
);
