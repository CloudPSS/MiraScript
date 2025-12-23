import { VmError } from '../../helpers/error.js';
import { display } from '../../helpers/serialize.js';
import { isVmFunction, isVmExtern, isVmConst } from '../../helpers/types.js';
import type { VmAny, VmArray, VmValue } from '../types/index.js';
import { $AssertInit } from './common.js';

export const $Call = (func: VmValue, args: readonly VmAny[]): VmValue => {
    for (const a of args) {
        $AssertInit(a);
    }
    if (isVmExtern(func)) {
        return func.call(args as readonly VmValue[]) ?? null;
    }
    if (isVmFunction(func)) {
        return func(...(args as readonly VmValue[])) ?? null;
    }
    throw new VmError(`Value is not callable: ${display(func)}`, null);
};

/** 过滤剩余参数数组 */
export const $VArgs = (varags: VmAny[]): VmArray => {
    for (let i = 0, l = varags.length; i < l; i++) {
        const el = varags[i];
        if (!isVmConst(el)) {
            varags[i] = null;
        }
    }
    return varags as VmArray;
};
