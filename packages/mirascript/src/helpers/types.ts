import type {
    VmAny,
    VmArray,
    VmConst,
    VmContext,
    VmExtern,
    VmFunction,
    VmFunctionInfo,
    VmFunctionLike,
    VmImmutable,
    VmModule,
    VmPrimitive,
    VmRecord,
    VmScript,
    VmValue,
} from '../vm/types/index.js';
import type { VmWrapper } from '../vm/types/wrapper.js';
import {
    kVmContext,
    kVmExtern,
    kVmFunction,
    kVmModule,
    kVmScript,
    kVmWrapper,
    VM_ARRAY_MAX_LENGTH,
} from './constants.js';
import { getPrototypeOf, isArray, keys, values } from './utils.js';

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
    if (value == null || typeof value !== 'object') return false;
    if (isVmWrapper(value)) return false;
    if (isVmArray(value)) return false;
    value satisfies VmRecord;
    return true;
}

/** 检查是否为仅包含从 0 开始的连续数字键的 MiraScript 记录 */
export function isVmArrayLikeRecordByEntires(entries: ReadonlyArray<readonly [string, unknown]>): boolean {
    const { length } = entries;
    if (length === 0) return true;
    if (length > VM_ARRAY_MAX_LENGTH) return false;
    const firstKey = entries[0]![0];
    if (firstKey !== '0') return false;
    const lastKey = entries[length - 1]![0];
    if (lastKey !== String(length - 1)) return false;
    return true;
}

/** 检查是否为仅包含从 0 开始的连续数字键的 MiraScript 记录 */
export function isVmArrayLikeRecordByKeys(keys: readonly string[]): boolean {
    const { length } = keys;
    if (length === 0) return true;
    if (length > VM_ARRAY_MAX_LENGTH) return false;
    const firstKey = keys[0]!;
    if (firstKey !== '0') return false;
    const lastKey = keys[length - 1]!;
    if (lastKey !== String(length - 1)) return false;
    return true;
}

/** 检查是否为仅包含从 0 开始的连续数字键的 MiraScript 记录 */
export function isVmArrayLikeRecord(value: VmRecord): boolean {
    return isVmArrayLikeRecordByKeys(keys(value));
}

const MAX_DEPTH = 16;
/**
 * 检查是否为 Mirascript 数组
 */
function isVmArrayDeep(value: readonly unknown[], depth: number): value is VmArray {
    // VmArray 应为普通数组
    // Array.prototype
    const proto1: unknown = getPrototypeOf(value);
    if (!isArray(proto1)) return false;
    if (!depth) return true;
    return value.every((item) => isVmConstInner(item, depth));
}
/**
 * 检查是否为 Mirascript 记录
 */
function isVmRecordDeep(value: object, depth: number): value is VmRecord {
    // VmRecord 应为普通对象或空原型对象
    let isRecord;
    // Object.prototype
    const proto1: unknown = getPrototypeOf(value);
    if (proto1 == null) {
        isRecord = true;
    } else {
        // null
        const proto2: unknown = getPrototypeOf(proto1);
        if (proto2 != null) {
            isRecord = false;
        } else {
            isRecord = 'hasOwnProperty' in value;
        }
    }
    if (!isRecord) return false;
    if (!depth) return true;
    return values(value).every((value) => isVmConstInner(value, depth));
}
/**
 * 检查是否为 Mirascript 值语义值
 */
function isVmConstInner(value: unknown, depth: number): value is VmConst {
    if (depth++ >= MAX_DEPTH) return false;
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
        case 'undefined': // undefined 在复合类型内部被视为 nil
            return true;
        case 'object':
            if (value == null) return true;
            if (isVmWrapper(value)) return false;
            if (isArray(value)) {
                return isVmArrayDeep(value, depth);
            } else {
                return isVmRecordDeep(value, depth);
            }
        case 'function':
        case 'bigint':
        case 'symbol':
        default:
            return false; // Other types are not valid
    }
}

/**
 * 检查是否为 Mirascript 值语义值
 */
export function isVmConst(value: VmAny): value is VmConst;
/**
 * 检查是否为 Mirascript 值语义值
 */
export function isVmConst(value: unknown, checkDeep: boolean): value is VmConst;
/**
 * 检查是否为 Mirascript 值语义值
 */
export function isVmConst(value: unknown, checkDeep = false): value is VmConst {
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
            return true;
        case 'object':
            if (value == null) return true;
            if (isVmWrapper(value)) return false;
            if (!checkDeep) {
                if (isArray(value)) {
                    return isVmArrayDeep(value, 0);
                } else {
                    return isVmRecordDeep(value, 0);
                }
            } else {
                return isVmConstInner(value, 1);
            }
        case 'undefined':
        case 'function':
        case 'bigint':
        case 'symbol':
        default:
            return false; // Other types are not valid
    }
}

/**
 * 检查是否为 Mirascript 不可变值
 */
export function isVmImmutable(value: VmAny): value is VmImmutable;
/**
 * 检查是否为 Mirascript 不可变值
 */
export function isVmImmutable(value: unknown, checkDeep: boolean): value is VmImmutable;
/**
 * 检查是否为 Mirascript 不可变值
 */
export function isVmImmutable(value: unknown, checkDeep = false): value is VmImmutable {
    return isVmModule(value) || isVmFunction(value) || isVmConst(value, checkDeep);
}

/** 检查是否为 Mirascript 值 */
export function isVmAny(value: unknown, checkDeep: boolean): value is VmAny {
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
        case 'undefined':
            return true;
        case 'object':
            if (value == null) return true;
            if (isVmWrapper(value)) return true;
            return isVmConst(value, checkDeep);
        case 'function':
            return isVmFunction(value);
        case 'bigint':
        case 'symbol':
        default:
            return false; // Other types are not valid
    }
}

/** 检查是否为 Mirascript 合法值 */
export function isVmValue(value: VmAny): value is VmValue;
/** 检查是否为 Mirascript 合法值 */
export function isVmValue(value: unknown, checkDeep: boolean): value is VmValue;
/** 检查是否为 Mirascript 合法值 */
export function isVmValue(value: unknown, checkDeep = false): value is VmValue {
    if (value === undefined) return false;
    return isVmAny(value, checkDeep);
}
