import { Cp } from '../helpers';
import { $CallDyn, $ToNumber } from '../operations';
import {
    isVmArray,
    isVmConst,
    isVmExtern,
    isVmPrimitive,
    isVmRecord,
    VmExtern,
    type VmArray,
    type VmConst,
    type VmRecord,
    type VmValue,
} from '../types/index.js';
import type { VmFunctionLike } from '../types/function.js';
import { VmError } from '../error';

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

export const map: VmFunctionLike = (data, fn) => {
    if (isVmPrimitive(data)) {
        return $CallDyn(fn, [data, null, data]);
    }
    if (isVmArray(data)) {
        return data.map((item: VmConst, index: number, arr: VmArray) => {
            Cp();
            const ret = $CallDyn(fn, [item, index, arr]);
            if (!isVmConst(ret)) return null;
            return ret;
        });
    }
    if (isVmRecord(data)) {
        return Object.fromEntries(
            Object.entries(data).map(([key, value]) => {
                Cp();
                const ret = $CallDyn(fn, [value, key, data]);
                if (!isVmConst(ret)) return [key, null] as const;
                return [key, ret] as const;
            }),
        );
    }
    if (isVmExtern(data)) {
        if (Array.isArray(data.value)) {
            let isConst = true;
            const result: VmValue[] = [];
            const { length } = data.value;
            for (let i = 0; i < length; i++) {
                Cp();
                const ret = $CallDyn(fn, [data.get(String(i)), i, data]);
                result.push(ret);
                if (!isVmConst(ret)) {
                    isConst = false;
                }
            }
            if (isConst) return result as VmArray;
            return new VmExtern(result);
        }
        let isConst = true;
        const result: Array<[string, VmValue]> = [];
        for (const key of data.keys()) {
            Cp();
            const ret = $CallDyn(fn, [data.get(key), key, data]);
            if (!isVmConst(ret)) {
                isConst = false;
            }
            result.push([key, ret]);
        }
        const obj = Object.fromEntries(result);
        if (isConst) return obj as VmRecord;
        return new VmExtern(obj);
    }
    throw new VmError('First argument must be primitive, array, record, or extern');
};

export const len: VmFunctionLike = (arr) => {
    if (!isVmArray(arr)) {
        throw new VmError('First argument must be an array');
    }
    return arr.length;
};
