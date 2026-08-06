import type { VmScript } from '../../compiler/index.js';
import type {
    VmAny,
    VmArray,
    VmContext,
    VmExtern,
    VmFunction,
    VmFunctionInfo,
    VmFunctionLike,
    VmImmutable,
    VmModule,
    VmPrimitive,
    VmRecord,
} from '../../vm/types/index.js';
import type { VmWrapper } from '../../vm/types/wrapper.js';
import { kVmContext, kVmExtern, kVmFunction, kVmModule, kVmScript, kVmWrapper } from '../constants.js';
import { isArray } from '../utils.js';

/** 检查是否为 Mirascript 脚本 */
export function isVmScript(value: unknown): value is VmScript {
    return typeof value === 'function' && kVmScript in value;
}

/** 检查是否为执行上下文 */
export function isVmContext(context: unknown): context is VmContext {
    return context != null && typeof context == 'object' && kVmContext in context;
}
/** 检查是否为 Mirascript 函数 */
export function isVmFunction<T extends VmFunctionLike>(value: unknown): value is VmFunction<T> {
    return typeof value == 'function' && kVmFunction in value;
}
/** 检查是否为 Mirascript 函数，并获取其信息 */
export function getVmFunctionInfo(value: unknown): VmFunctionInfo | undefined {
    if (typeof value != 'function') return undefined;
    return (value as VmFunction)[kVmFunction];
}
/** 检查值是否为 MiraScript 包装器 */
export function isVmWrapper<T extends object>(value: unknown): value is VmWrapper<T> {
    return value != null && typeof value == 'object' && kVmWrapper in value;
}
/** 检查值是否为 Mirascript 模块 */
export function isVmModule<T extends Record<string, VmImmutable>>(value: unknown): value is VmModule<T> {
    return value != null && typeof value == 'object' && kVmModule in value;
}
/** 检查值是否为 Mirascript 外部值 */
export function isVmExtern<T extends object>(value: unknown): value is VmExtern<T> {
    return value != null && typeof value == 'object' && kVmExtern in value;
}

/** 检查值是否为 Mirascript 可调用值 */
// eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
export function isVmCallable<E extends Function, F extends VmFunctionLike>(
    value: unknown,
): value is VmFunction<F> | VmExtern<E> {
    return isVmFunction<F>(value) || (isVmExtern<E>(value) && typeof value.value == 'function');
}

/** 检查值是否为 Mirascript 原始值 */
export function isVmPrimitive(value: unknown): value is VmPrimitive {
    if (value === null || typeof value == 'number' || typeof value == 'string' || typeof value == 'boolean') {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
        value as VmPrimitive satisfies typeof value;
        value satisfies VmPrimitive;
        return true;
    }
    return false;
}

/** 检查值是否为 Mirascript 数组 */
export function isVmArray(value: VmAny): value is VmArray {
    if (!isArray(value)) return false;
    value as VmArray satisfies VmArray;
    return true;
}

/** 检查值是否为 Mirascript 记录 */
export function isVmRecord(value: VmAny): value is VmRecord {
    if (value == null || typeof value != 'object') return false;
    if (isVmWrapper(value) || isVmArray(value)) return false;
    value satisfies VmRecord;
    return true;
}
