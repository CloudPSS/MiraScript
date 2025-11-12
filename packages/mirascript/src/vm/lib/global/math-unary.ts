import type { VmAny } from '../../index.js';
import { $ToNumber } from '../../operations.js';
import { required, VmLib } from '../_helpers.js';

/** 生成函数 */
function build(f: (x: number) => number): (x: VmAny) => number {
    return (x) => {
        required('x', x, Number.NaN);
        return f($ToNumber(x));
    };
}

export const trunc = VmLib(build(Math.trunc), {
    summary: '返回数值的整数部分（去除小数）',
    params: { x: '要取整数部分的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const floor = VmLib(build(Math.floor), {
    summary: '返回小于等于给定数的最大整数',
    params: { x: '要向下取整的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const ceil = VmLib(build(Math.ceil), {
    summary: '返回大于等于给定数的最小整数',
    params: { x: '要向上取整的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});

const _round = Math.round;
const _abs = Math.abs;
export const round = VmLib(
    build((x) => {
        // Ref: https://github.com/python/cpython/blob/9ce99c6c1901705238e4cb3ce81eb6f499e7b4f4/Objects/floatobject.c#L1052-L1060
        let rounded = _round(x);
        if (_abs(x - rounded) === 0.5) {
            // 使用银行家舍入法
            rounded = 2 * _round(x / 2);
        }
        return rounded;
    }),
    {
        summary: '返回四舍五入后的整数',
        params: { x: '要四舍五入的数' },
        paramsType: { x: 'number' },
        returnsType: 'number',
    },
);
export const sign = VmLib(build(Math.sign), {
    summary: '返回数值的符号（正数为 1，负数为 -1，零为 0）',
    params: { x: '要判断符号的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const abs = VmLib(build(Math.abs), {
    summary: '返回数值的绝对值',
    params: { x: '要取绝对值的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});

export const acos = VmLib(build(Math.acos), {
    summary: '返回数值的反余弦值（弧度）',
    params: { x: '要计算反余弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const acosh = VmLib(build(Math.acosh), {
    summary: '返回数值的反双曲余弦值',
    params: { x: '要计算反双曲余弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const asin = VmLib(build(Math.asin), {
    summary: '返回数值的反正弦值（弧度）',
    params: { x: '要计算反正弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const asinh = VmLib(build(Math.asinh), {
    summary: '返回数值的反双曲正弦值',
    params: { x: '要计算反双曲正弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const atan = VmLib(build(Math.atan), {
    summary: '返回数值的反正切值（弧度）',
    params: { x: '要计算反正切的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const atanh = VmLib(build(Math.atanh), {
    summary: '返回数值的反双曲正切值',
    params: { x: '要计算反双曲正切的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const cos = VmLib(build(Math.cos), {
    summary: '返回数值的余弦值',
    params: { x: '要计算余弦的数（弧度）' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const cosh = VmLib(build(Math.cosh), {
    summary: '返回数值的双曲余弦值',
    params: { x: '要计算双曲余弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const sin = VmLib(build(Math.sin), {
    summary: '返回数值的正弦值',
    params: { x: '要计算正弦的数（弧度）' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const sinh = VmLib(build(Math.sinh), {
    summary: '返回数值的双曲正弦值',
    params: { x: '要计算双曲正弦的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const tan = VmLib(build(Math.tan), {
    summary: '返回数值的正切值',
    params: { x: '要计算正切的数（弧度）' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const tanh = VmLib(build(Math.tanh), {
    summary: '返回数值的双曲正切值',
    params: { x: '要计算双曲正切的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});

export const exp = VmLib(build(Math.exp), {
    summary: '返回 e 的指定次幂',
    params: { x: '指数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const expm1 = VmLib(build(Math.expm1), {
    summary: '返回 e 的 x 次幂减 1',
    params: { x: '指数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const log = VmLib(build(Math.log), {
    summary: '返回数值的自然对数（以 e 为底）',
    params: { x: '要取对数的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const log10 = VmLib(build(Math.log10), {
    summary: '返回数值的以 10 为底的对数',
    params: { x: '要取对数的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const log1p = VmLib(build(Math.log1p), {
    summary: '返回 1 加上数值的自然对数',
    params: { x: '要取对数的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const log2 = VmLib(build(Math.log2), {
    summary: '返回数值的以 2 为底的对数',
    params: { x: '要取对数的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});

export const sqrt = VmLib(build(Math.sqrt), {
    summary: '返回数值的平方根',
    params: { x: '要开平方的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
export const cbrt = VmLib(build(Math.cbrt), {
    summary: '返回数值的立方根',
    params: { x: '要计算立方根的数' },
    paramsType: { x: 'number' },
    returnsType: 'number',
});
