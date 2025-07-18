import { $ToNumber } from '../../operations.js';
import { type VmAny, isVmArray } from '../../types/index.js';
import { VmLib } from '../helpers.js';

/** Get the minimum and maximum numbers from the arguments. */
function getMinMaxNumbers(args: readonly VmAny[]): number[] {
    if (args.length === 0) return [];
    if (args.length === 1 && isVmArray(args[0])) args = args[0];
    const numbers: number[] = [];
    for (const arg of args) {
        if (arg == null) continue;
        numbers.push($ToNumber(arg));
    }
    return numbers;
}

/** 生成函数 */
function build(f: (...values: readonly number[]) => number): (...values: readonly VmAny[]) => number {
    return (...values) => {
        const numbers = getMinMaxNumbers(values);
        return f(...numbers);
    };
}

export const max = VmLib(build(Math.max), {
    summary: '返回一组数中的最大值',
    params: { '..values': '要比较的数值' },
    paramsType: { '..values': '[number]' },
    returnsType: 'number',
});

export const min = VmLib(build(Math.min), {
    summary: '返回一组数中的最小值',
    params: { '..values': '要比较的数值' },
    paramsType: { '..values': '[number]' },
    returnsType: 'number',
});

export const hypot = VmLib(build(Math.hypot), {
    summary: '返回所有参数平方和的平方根',
    params: { '..values': '要计算的数值' },
    paramsType: { '..values': '[number]' },
    returnsType: 'number',
});
