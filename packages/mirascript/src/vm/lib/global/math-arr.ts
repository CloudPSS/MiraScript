import type { VmAny } from '../../types/index.ts';
import { getNumbers, VmLib } from '../_helpers.ts';

/** 生成函数 */
function build(f: (...values: readonly number[]) => number): (...values: readonly VmAny[]) => number {
    return (...values) => {
        const numbers = getNumbers(values);
        return f(...numbers);
    };
}

export const max = VmLib(build(Math.max), {
    summary: '返回一组数中的最大值',
    params: { '..values': '要比较的数值' },
    paramsType: { '..values': 'number[]' },
    returnsType: 'number',
    examples: ['max(3, 7, 2) // 7'],
});

export const min = VmLib(build(Math.min), {
    summary: '返回一组数中的最小值',
    params: { '..values': '要比较的数值' },
    paramsType: { '..values': 'number[]' },
    returnsType: 'number',
    examples: ['min(3, 7, 2) // 2'],
});

export const hypot = VmLib(build(Math.hypot), {
    summary: '返回所有参数平方和的平方根',
    params: { '..values': '要计算的数值' },
    paramsType: { '..values': 'number[]' },
    returnsType: 'number',
    examples: ['hypot(3, 4) // 5'],
});

export const sum = VmLib(
    (...values: readonly VmAny[]) => {
        const numbers = getNumbers(values);
        return numbers.reduce((a, b) => a + b, 0);
    },
    {
        summary: '返回一组数的总和',
        params: { '..values': '要计算的数值' },
        paramsType: { '..values': 'number[]' },
        returnsType: 'number',
        examples: ['sum(1, 2, 3, 4) // 10'],
    },
);

export const product = VmLib(
    (...values: readonly VmAny[]) => {
        const numbers = getNumbers(values);
        return numbers.reduce((a, b) => a * b, 1);
    },
    {
        summary: '返回一组数的乘积',
        params: { '..values': '要计算的数值' },
        paramsType: { '..values': 'number[]' },
        returnsType: 'number',
        examples: ['product(2, 3, 4) // 24'],
    },
);
