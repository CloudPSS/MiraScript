import { create, defineProperty, entries, isFinite } from '../../helpers/utils.js';
import { VM_ARRAY_MAX_LENGTH, VM_FUNCTION_ANONYMOUS_NAME } from '../../helpers/constants.js';
import { isVmConst } from '../../helpers/types.js';
import type { VmFunctionLike } from '../types/function.js';
import { DefaultVmContext, type VmContext } from '../types/context.js';
import type { VmConst, VmAny, VmArray, VmValue, VmImmutable } from '../types/index.js';
import { VmModule } from '../types/module.js';
import { VmFunction } from '../types/function.js';
import { $AssertInit } from './common.js';
import { $ToNumber } from './convert.js';

/** 构造 module */
export function $Module<T extends Record<string, () => VmImmutable>>(
    name: string,
    body: T,
): VmModule<{ [K in keyof T]: ReturnType<T[K]> }> {
    const mod = create(null) as { [K in keyof T]: ReturnType<T[K]> };
    for (const [key, get] of entries(body)) {
        defineProperty(mod, key, { __proto__: null, get, enumerable: true });
    }
    return new VmModule(name, mod);
}

/** 构造 record | array 元素 */
export function $El(value: VmAny): VmConst {
    $AssertInit(value);
    if (!isVmConst(value)) return null;
    return value;
}

const EMPTY = create(null);
/** 构造 record 可选元素 */
export function $ElOpt(key: string, value: VmAny): VmConst {
    $AssertInit(value);
    if (value == null || !isVmConst(value)) return EMPTY;
    return { __proto__: null, [key]: value };
}

/** 构造函数和函数表达式 */
export function $Fn<T extends VmFunctionLike>(name: string | null | undefined, fn: T): VmFunction<T> {
    return VmFunction(fn, { isLib: false, name: name || VM_FUNCTION_ANONYMOUS_NAME });
}

/** 读取闭包上值 */
export function $Upvalue<T extends VmValue>(value: T | undefined): T {
    $AssertInit(value);
    return value;
}

const assertArrayLength = (len: number) => {
    if (len > VM_ARRAY_MAX_LENGTH) {
        throw new RangeError(`Array length exceeds maximum limit of ${VM_ARRAY_MAX_LENGTH}`);
    }
};
const isEmptyRange = (start: number, end: number) => {
    return !isFinite(start) || !isFinite(end) || start > end;
};
/** 构造范围数组 */
export function $ArrayRange(start: VmAny, end: VmAny): VmArray {
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    if (isEmptyRange(s, e)) return [];
    assertArrayLength(e - s + 1);
    const arr = [];
    for (let i = s; i <= e; i++) {
        arr.push(i);
    }
    return arr;
}
/** 构造范围数组（不包含结束值） */
export function $ArrayRangeExclusive(start: VmAny, end: VmAny): VmArray {
    const s = $ToNumber(start);
    const e = $ToNumber(end);
    if (isEmptyRange(s, e)) return [];
    assertArrayLength(e - s);
    const arr = [];
    for (let i = s; i < e; i++) {
        arr.push(i);
    }
    return arr;
}

/** 默认执行上下文 */
export function $GlobalFallback(): VmContext {
    return DefaultVmContext;
}
