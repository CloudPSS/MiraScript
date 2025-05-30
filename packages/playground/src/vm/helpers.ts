import { VmError } from './error.js';
import { $ToNumber } from './operations.js';
import { VmFunction, type VmAny, type VmArray, type VmRecord } from './types/index.js';
import type { Mutable } from './utils.js';

export const Function = (fn: (...args: VmAny[]) => VmAny): VmFunction => {
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
export const RecordFreeze = (record: Mutable<VmRecord>, optional: readonly string[]): void => {
    for (const field of optional) {
        if (record[field] == null) {
            delete record[field];
        }
    }
};

let cp = Number.NaN;
let cpTimeout = 100; // Default timeout in milliseconds
/** 检查点 */
export function Cp(): void {
    if (!cp) {
        cp = Date.now();
    } else if (Date.now() - cp > cpTimeout) {
        throw new VmError('Execution timeout');
    }
}
/** 清除检查点超时时间 */
export function clearCheckpoint(): void {
    cp = Number.NaN;
}
/** 设置检查点超时时间 */
export function configCheckpoint(timeout?: number): void {
    cpTimeout = timeout ?? 100;
}
