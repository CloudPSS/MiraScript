import type { VmAny } from '../../../index.js';
import { VmLib, expectNumber } from '../helpers.js';

/** 生成函数 */
function build(
    f: (x: number, y: number) => number,
    summary: string,
    examples: string[],
    y = '第二个操作数',
): VmLib<(str: VmAny, search: VmAny) => number> {
    return VmLib(
        (x, y) => {
            const a = expectNumber(0, x);
            const b = expectNumber(1, y);
            return f(a, b);
        },
        {
            summary,
            params: {
                x: { type: 'number', description: '第一个操作数' },
                y: { type: 'number', description: y },
            },
            returns: { type: 'number' },
            examples,
        },
    );
}

export const b_and = build((x, y) => x & y, '返回两个数的按位与', ['b_and(6, 3) // 2']);

export const b_or = build((x, y) => x | y, '返回两个数的按位或', ['b_or(5, 2) // 7']);

export const b_xor = build((x, y) => x ^ y, '返回两个数的按位异或', ['b_xor(5, 3) // 6']);

export const b_not = VmLib(
    (x) => {
        return ~expectNumber('x', x);
    },
    {
        summary: '返回一个数的按位取反',
        params: { x: { type: 'number', description: '操作数' } },
        returns: { type: 'number' },
        examples: ['b_not(0) // -1'],
    },
);

export const shl = build((x, y) => x << y, '返回第一个操作数左移指定的位数', ['shl(3, 2) // 12'], '位数');

export const sar = build((x, y) => x >> y, '返回第一个操作数右移指定的位数', ['sar(-8, 1) // -4'], '位数');

export const shr = build((x, y) => x >>> y, '返回第一个操作数无符号右移指定的位数', ['shr(8, 1) // 4'], '位数');
