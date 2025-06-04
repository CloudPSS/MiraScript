import { $Init, $ToNumber } from './operations.js';
import type { VmFunctionLike } from './types/function.js';
import { createVmGlobal, type VmGlobal } from './types/global.js';
import { isVmConst, VmFunction, type VmConst, type VmAny, type VmArray, type VmRecord } from './types/index.js';

export const Element = (value: VmAny): VmConst => {
    $Init(value);
    if (!isVmConst(value)) return null;
    return value;
};

export const Function = (fn: VmFunctionLike): VmFunction => {
    return VmFunction(fn, { isLib: false, injectCp: false });
};

export const ArrayRange = (start: VmAny, end: VmAny): VmArray => {
    const arr = [];
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    for (let i = s; i <= e; i++) {
        arr.push(i);
    }
    return arr;
};
export const ArrayRangeExclusive = (start: VmAny, end: VmAny): VmArray => {
    const arr = [];
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    for (let i = s; i < e; i++) {
        arr.push(i);
    }
    return arr;
};
export const RecordFreeze = (record: Writable<VmRecord>, optional: readonly string[]): void => {
    for (const field of optional) {
        if (record[field] == null) {
            delete record[field];
        }
    }
};

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
    if (cpDepth <= 0) {
        cp = Date.now();
        cpDepth = 0;
    }
    cpDepth++;
}
/** 检查点 */
export function CpExit(): void {
    Cp();
    cpDepth--;
    if (cpDepth <= 0) {
        cp = Number.NaN;
        cpDepth = 0;
    }
}
/** 设置检查点超时时间 */
export function configCheckpoint(timeout?: number): void {
    cpTimeout = timeout ?? 100;
}
/** 默认全局环境 */
export function GlobalFallback(): VmGlobal {
    return createVmGlobal();
}
