import type { VmAny } from '../../../types/index.js';
import { expectNumber, VmLib } from '../../helpers.js';
import * as m from './math.js';
import * as b from './basic.js';
import { record, unary, binary, scalar } from './utils.js';

export const I = VmLib(record([0, 1]), { summary: '虚数单位 (0, 1)' });
export const from = unary((z) => z, '按数字转换语义将输入转换为 (实部, 虚部)，忽略额外字段');
export const real = scalar((z) => z[0], '返回复数的实部');
export const imag = scalar((z) => z[1], '返回复数的虚部');
export const abs = scalar((z) => Math.hypot(...z), '返回复数的模');
export const arg = scalar((z) => Math.atan2(z[1], z[0]), '返回复数的主辐角（弧度，保留符号零）');
export const conj = unary(b.conj, '返回共轭复数');
export const neg = unary(b.neg, '返回复数的相反数');
export const add = binary(b.add, '返回两个复数的和');
export const subtract = binary(b.subtract, '返回两个复数的差');
export const multiply = binary(b.multiply, '返回两个复数的积');
export const divide = binary(b.divide, '返回两个复数的商');
export const pow = binary(m.power, '返回复数幂的主值');
export const sqrt = unary(m.squareRoot, '返回复数平方根的主值');
export const cbrt = unary(m.cubeRoot, '返回复数立方根的主值');
export const exp = unary(m.exponential, '返回复数指数');
export const expm1 = unary(m.exponentialMinusOne, '返回复数指数减一，保留近零精度');
export const log = unary(m.logarithm, '返回复数自然对数的主值');
export const log1p = unary(m.logarithmOnePlus, '返回一加复数的自然对数主值，保留近零精度');
export const log2 = unary((z) => m.logBase(z, Math.LN2), '返回以 2 为底的复数对数主值');
export const log10 = unary((z) => m.logBase(z, Math.LN10), '返回以 10 为底的复数对数主值');
export const sin = unary(m.sine, '返回复数正弦');
export const cos = unary(m.cosine, '返回复数余弦');
export const tan = unary(m.tangent, '返回复数正切');
export const sinh = unary(m.hyperbolicSine, '返回复数双曲正弦');
export const cosh = unary(m.hyperbolicCosine, '返回复数双曲余弦');
export const tanh = unary(m.hyperbolicTangent, '返回复数双曲正切');
export const asin = unary(m.arcSine, '返回复数反正弦主值');
export const acos = unary(m.arcCosine, '返回复数反余弦主值');
export const atan = unary(m.arcTangent, '返回复数反正切主值');
export const asinh = unary(m.areaHyperbolicSine, '返回复数反双曲正弦主值');
export const acosh = unary(m.areaHyperbolicCosine, '返回复数反双曲余弦主值');
export const atanh = unary(m.areaHyperbolicTangent, '返回复数反双曲正切主值');
export const polar = VmLib(
    (r: VmAny, theta: VmAny) => record(b.polar(expectNumber('r', r), expectNumber('theta', theta))),
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
