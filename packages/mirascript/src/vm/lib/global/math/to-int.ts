import type { VmAny } from '../../../index.js';
import { expectIntegerRange, expectNumber, VmLib } from '../../helpers.js';

/** 生成函数 */
function build(f: (x: number) => number): (x: VmAny, n: VmAny) => number {
    return (x, n) => {
        const xn = expectNumber('x', x);
        const nn = n != null ? expectIntegerRange('n', n, 0, 15) : 0;
        if (!nn) return f(xn);
        const factor = 10 ** nn;
        return f(xn * factor) / factor;
    };
}

export const trunc = VmLib(build(Math.trunc), {
    summary: '向零方向舍入数值',
    params: {
        x: { type: 'number', description: '要舍入的数' },
        n: { type: 'number | nil', description: '小数位数，默认为 0' },
    },
    returns: { type: 'number' },
});
export const floor = VmLib(build(Math.floor), {
    summary: '向下舍入数值',
    params: {
        x: { type: 'number', description: '要舍入的数' },
        n: { type: 'number | nil', description: '小数位数，默认为 0' },
    },
    returns: { type: 'number' },
});
export const ceil = VmLib(build(Math.ceil), {
    summary: '向上舍入数值',
    params: {
        x: { type: 'number', description: '要舍入的数' },
        n: { type: 'number | nil', description: '小数位数，默认为 0' },
    },
    returns: { type: 'number' },
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
        summary: '四舍五入数值',
        params: {
            x: { type: 'number', description: '要舍入的数' },
            n: { type: 'number | nil', description: '小数位数，默认为 0' },
        },
        returns: { type: 'number' },
    },
);
