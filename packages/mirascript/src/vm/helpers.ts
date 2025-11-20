import { isFinite } from '../helpers/utils.js';
import { VM_ARRAY_MAX_LENGTH } from '../helpers/constants.js';
import { isVmConst } from '../helpers/types.js';
import { $AssertInit, $ToNumber } from './operations.js';
import type { VmFunctionLike } from './types/function.js';
import { DefaultVmContext, type VmContext } from './types/context.js';
import type { VmConst, VmAny, VmArray, VmValue } from './types/index.js';
import { VmFunction } from './types/function.js';

export { Cp, CpEnter, CpExit } from './checkpoint.js';

export const Vargs = (varags: VmAny[]): VmArray => {
    for (let i = 0, l = varags.length; i < l; i++) {
        const el = varags[i];
        if (!isVmConst(el)) {
            varags[i] = null;
        }
    }
    return varags as VmArray;
};
export const Element = (value: VmAny): VmConst => {
    $AssertInit(value);
    if (!isVmConst(value)) return null;
    return value;
};

export const ElementOpt = (key: string, value: VmAny): VmConst => {
    $AssertInit(value);
    if (value == null || !isVmConst(value)) return {};
    return { [key]: value };
};

export const Function = (fn: VmFunctionLike): VmFunction => {
    return VmFunction(fn, { isLib: false, injectCp: false });
};

export const Upvalue = (value: VmAny): VmValue => {
    $AssertInit(value);
    return value;
};

const assertArrayLength = (start: number, end: number) => {
    if (end - start > VM_ARRAY_MAX_LENGTH) {
        throw new RangeError(`Array length exceeds maximum limit of ${VM_ARRAY_MAX_LENGTH}`);
    }
};
const isEmptyRange = (start: number, end: number) => {
    return !isFinite(start) || !isFinite(end) || start > end;
};
export const ArrayRange = (start: VmAny, end: VmAny): VmArray => {
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    if (isEmptyRange(s, e)) return [];
    assertArrayLength(s, e);
    const arr = [];
    for (let i = s; i <= e; i++) {
        arr.push(i);
    }
    return arr;
};
export const ArrayRangeExclusive = (start: VmAny, end: VmAny): VmArray => {
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    if (isEmptyRange(s, e)) return [];
    assertArrayLength(s, e);
    const arr = [];
    for (let i = s; i < e; i++) {
        arr.push(i);
    }
    return arr;
};

/** 默认执行上下文 */
export function GlobalFallback(): VmContext {
    return DefaultVmContext;
}
