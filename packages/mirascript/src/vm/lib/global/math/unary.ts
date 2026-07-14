import type { VmAny } from '../../../index.js';
import { expectNumber, VmLib } from '../../helpers.js';

/** 生成函数 */
function build(f: (x: number) => number): (x: VmAny) => number {
    return (x) => {
        return f(expectNumber('x', x));
    };
}

export const sign = VmLib(build(Math.sign), {
    summary: '返回数值的符号（正数为 1，负数为 -1，零为 0）',
    params: { x: { type: 'number', description: '要判断符号的数' } },
    returns: { type: 'number' },
});
export const abs = VmLib(build(Math.abs), {
    summary: '返回数值的绝对值',
    params: { x: { type: 'number', description: '要取绝对值的数' } },
    returns: { type: 'number' },
});

export const acos = VmLib(build(Math.acos), {
    summary: '返回数值的反余弦值（弧度）',
    params: { x: { type: 'number', description: '要计算反余弦的数' } },
    returns: { type: 'number' },
});
export const acosh = VmLib(build(Math.acosh), {
    summary: '返回数值的反双曲余弦值',
    params: { x: { type: 'number', description: '要计算反双曲余弦的数' } },
    returns: { type: 'number' },
});
export const asin = VmLib(build(Math.asin), {
    summary: '返回数值的反正弦值（弧度）',
    params: { x: { type: 'number', description: '要计算反正弦的数' } },
    returns: { type: 'number' },
});
export const asinh = VmLib(build(Math.asinh), {
    summary: '返回数值的反双曲正弦值',
    params: { x: { type: 'number', description: '要计算反双曲正弦的数' } },
    returns: { type: 'number' },
});
export const atan = VmLib(build(Math.atan), {
    summary: '返回数值的反正切值（弧度）',
    params: { x: { type: 'number', description: '要计算反正切的数' } },
    returns: { type: 'number' },
});
export const atanh = VmLib(build(Math.atanh), {
    summary: '返回数值的反双曲正切值',
    params: { x: { type: 'number', description: '要计算反双曲正切的数' } },
    returns: { type: 'number' },
});
export const cos = VmLib(build(Math.cos), {
    summary: '返回数值的余弦值',
    params: { x: { type: 'number', description: '要计算余弦的数（弧度）' } },
    returns: { type: 'number' },
});
export const cosh = VmLib(build(Math.cosh), {
    summary: '返回数值的双曲余弦值',
    params: { x: { type: 'number', description: '要计算双曲余弦的数' } },
    returns: { type: 'number' },
});
export const sin = VmLib(build(Math.sin), {
    summary: '返回数值的正弦值',
    params: { x: { type: 'number', description: '要计算正弦的数（弧度）' } },
    returns: { type: 'number' },
});
export const sinh = VmLib(build(Math.sinh), {
    summary: '返回数值的双曲正弦值',
    params: { x: { type: 'number', description: '要计算双曲正弦的数' } },
    returns: { type: 'number' },
});
export const tan = VmLib(build(Math.tan), {
    summary: '返回数值的正切值',
    params: { x: { type: 'number', description: '要计算正切的数（弧度）' } },
    returns: { type: 'number' },
});
export const tanh = VmLib(build(Math.tanh), {
    summary: '返回数值的双曲正切值',
    params: { x: { type: 'number', description: '要计算双曲正切的数' } },
    returns: { type: 'number' },
});

export const exp = VmLib(build(Math.exp), {
    summary: '返回 e 的指定次幂',
    params: { x: { type: 'number', description: '指数' } },
    returns: { type: 'number' },
});
export const expm1 = VmLib(build(Math.expm1), {
    summary: '返回 e 的 x 次幂减 1',
    params: { x: { type: 'number', description: '指数' } },
    returns: { type: 'number' },
});
export const log = VmLib(build(Math.log), {
    summary: '返回数值的自然对数（以 e 为底）',
    params: { x: { type: 'number', description: '要取对数的数' } },
    returns: { type: 'number' },
});
export const log10 = VmLib(build(Math.log10), {
    summary: '返回数值的以 10 为底的对数',
    params: { x: { type: 'number', description: '要取对数的数' } },
    returns: { type: 'number' },
});
export const log1p = VmLib(build(Math.log1p), {
    summary: '返回 1 加上数值的自然对数',
    params: { x: { type: 'number', description: '要取对数的数' } },
    returns: { type: 'number' },
});
export const log2 = VmLib(build(Math.log2), {
    summary: '返回数值的以 2 为底的对数',
    params: { x: { type: 'number', description: '要取对数的数' } },
    returns: { type: 'number' },
});

export const sqrt = VmLib(build(Math.sqrt), {
    summary: '返回数值的平方根',
    params: { x: { type: 'number', description: '要开平方的数' } },
    returns: { type: 'number' },
});
export const cbrt = VmLib(build(Math.cbrt), {
    summary: '返回数值的立方根',
    params: { x: { type: 'number', description: '要计算立方根的数' } },
    returns: { type: 'number' },
});
