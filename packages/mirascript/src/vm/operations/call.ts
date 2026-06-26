import { VmError } from '../../helpers/error.js';
import { display } from '../../helpers/serialize.js';
import { isVmExtern, isVmConst } from '../../helpers/types.js';
import type { VmExtern, VmFunction, VmAny, VmArray, VmValue } from '../types/index.js';
import { $AssertInit } from './common.js';

/** 调用返回类型 */
type CallReturn<T extends VmValue> =
    T extends VmFunction<infer F>
        ? F extends (...args: readonly VmAny[]) => infer R
            ? // eslint-disable-next-line @typescript-eslint/no-invalid-void-type
              R extends void | undefined
                ? null
                : R
            : VmValue
        : T extends VmExtern<infer E>
          ? E extends (...args: readonly VmAny[]) => infer R
              ? // eslint-disable-next-line @typescript-eslint/no-invalid-void-type
                R extends void | undefined
                  ? null
                  : R extends VmValue
                    ? R
                    : R extends object
                      ? VmExtern<R>
                      : VmValue
              : VmValue
          : VmValue;

/** 调用函数 */
export function $Call<T extends VmValue, A extends readonly VmValue[]>(func: T, args: A): CallReturn<T> {
    const argsLen = args.length;
    for (let i = 0; i < argsLen; i++) {
        $AssertInit(args[i]);
    }
    if (typeof func == 'function') {
        return (func(...args) ?? null) as CallReturn<T>;
    }
    if (isVmExtern(func)) {
        return (func.call(args) ?? null) as CallReturn<T>;
    }
    throw new VmError(`Value is not callable: ${display(func)}`, null);
}

/** 过滤剩余参数数组 */
export function $VArgs(varags: VmAny[]): VmArray {
    for (let i = 0, l = varags.length; i < l; i++) {
        const el = varags[i];
        if (!isVmConst(el)) {
            varags[i] = null;
        }
    }
    return varags as VmArray;
}
