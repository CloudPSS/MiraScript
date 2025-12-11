import { VmError } from '../../helpers/error.js';
import { display } from '../../helpers/serialize.js';
import { isVmFunction, isVmExtern } from '../../helpers/types.js';
import type { VmAny, VmValue } from '../types/index.js';
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
