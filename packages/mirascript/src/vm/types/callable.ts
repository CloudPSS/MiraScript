import { isVmExtern, type VmExtern } from './extern.js';
import { isVmFunction, type VmFunction, type VmFunctionLike } from './function.js';

/** 检查值是否为 Mirascript 可调用值 */
// eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
export function isVmCallable<E extends Function, F extends VmFunctionLike>(
    value: unknown,
): value is VmFunction<F> | VmExtern<E> {
    return isVmFunction<F>(value) || (isVmExtern<E>(value) && typeof value.value == 'function');
}
