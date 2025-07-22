import { $AssertInit, $ToBoolean, $ToNumber } from './operations.js';
import type { VmFunctionLike } from './types/function.js';
import { createVmContext, type VmContext } from './types/context.js';
import { isVmConst, VmFunction, type VmConst, type VmAny, type VmArray, type VmValue } from './types/index.js';
const { isFinite } = Number;
const { ceil } = Math;

export const Truthy = (value: VmAny): boolean => {
    // Optimize for boolean values
    if (typeof value == 'boolean') return value;
    return $ToBoolean(value);
};

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

export const ArrayRange = (start: VmAny, end: VmAny): VmArray => {
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    if (!isFinite(s) || !isFinite(e) || s > e) {
        return [];
    }
    const arr = [];
    for (let i = ceil(s); i <= e; i++) {
        arr.push(i);
    }
    return arr;
};
export const ArrayRangeExclusive = (start: VmAny, end: VmAny): VmArray => {
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    if (!isFinite(s) || !isFinite(e) || s > e) {
        return [];
    }
    const arr = [];
    for (let i = ceil(s); i < e; i++) {
        arr.push(i);
    }
    return arr;
};

const MAX_DEPTH = 128;

let cpDepth = 0;
let cp = Number.NaN;
let cpTimeout = 100; // Default timeout in milliseconds
/** 检查点 */
export function Cp(): void {
    if (!cp) {
        cp = Date.now();
    } else if (Date.now() - cp > cpTimeout) {
        throw new RangeError('Execution timeout');
    }
}
/** 检查点 */
export function CpEnter(): void {
    cpDepth++;
    if (cpDepth <= 1) {
        cp = Date.now();
        cpDepth = 1;
    } else if (cpDepth > MAX_DEPTH) {
        throw new RangeError('Maximum call depth exceeded');
    } else {
        Cp();
    }
}
/** 检查点 */
export function CpExit(): void {
    cpDepth--;
    if (cpDepth < 1) {
        cp = Number.NaN;
        cpDepth = 0;
    } else {
        Cp();
    }
}
/** 设置检查点超时时间 */
export function configCheckpoint(timeout?: number): void {
    if (typeof timeout !== 'number' || timeout <= 0 || Number.isNaN(timeout)) {
        throw new RangeError('Invalid timeout value');
    }
    cpTimeout = timeout ?? 100;
}
/** 默认执行上下文 */
export function GlobalFallback(): VmContext {
    return createVmContext();
}
