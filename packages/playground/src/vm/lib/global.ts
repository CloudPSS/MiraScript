import { Cp } from '../helpers';
import { $CallDyn, $ToNumber } from '../operations';
import { isVmArray, isVmConst, type VmArray, type VmConst, type VmValue } from '../types';
import type { VmFunctionLike } from '../types/function.js';

/** Get the minimum and maximum numbers from the arguments. */
function getMinMaxNumbers(args: VmValue[]): number[] {
    if (args.length === 0) return [];
    if (!isVmArray(args[0])) return args.map($ToNumber);
    return args[0].map($ToNumber);
}

export const max: VmFunctionLike = (...args) => {
    const numbers = getMinMaxNumbers(args);
    return Math.max(...numbers);
};
export const min: VmFunctionLike = (...args) => {
    const numbers = getMinMaxNumbers(args);
    return Math.min(...numbers);
};

export const map: VmFunctionLike = (arr, fn) => {
    if (!isVmArray(arr)) {
        throw new TypeError('First argument must be an array');
    }
    return arr.map((item: VmConst, index: number, arr: VmArray) => {
        Cp();
        const ret = $CallDyn(fn, [item, index, arr]);
        if (!isVmConst(ret)) return null;
        return ret;
    });
};
