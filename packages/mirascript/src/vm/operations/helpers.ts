import { create, defineProperty, entries } from '../../helpers/utils.js';
import { VM_FUNCTION_ANONYMOUS_NAME } from '../../helpers/constants.js';
import { isVmConst } from '../../helpers/types.js';
import type { VmFunctionLike } from '../types/function.js';
import { DefaultVmContext, type VmContext } from '../types/context.js';
import type { VmConst, VmAny, VmValue, VmImmutable } from '../types/index.js';
import { VmModule } from '../types/module.js';
import { VmFunction } from '../types/function.js';
import { $AssertInit } from './common.js';

/** 构造 module */
export const $Module = <T extends Record<string, () => VmImmutable>>(
    name: string,
    body: T,
): VmModule<{ [K in keyof T]: ReturnType<T[K]> }> => {
    const mod = create(null) as { [K in keyof T]: ReturnType<T[K]> };
    for (const [key, get] of entries(body)) {
        defineProperty(mod, key, { __proto__: null, get, enumerable: true, configurable: true });
    }
    return new VmModule(name, mod);
};

/** 构造 record | array 元素 */
export const $El = (value: VmAny): VmConst => {
    $AssertInit(value);
    if (!isVmConst(value)) return null;
    return value;
};

const EMPTY = create(null);
/** 构造 record 可选元素 */
export const $ElOpt = (key: string, value: VmAny): VmConst => {
    $AssertInit(value);
    if (value == null || !isVmConst(value)) return EMPTY;
    return { __proto__: null, [key]: value };
};

/** 构造函数和函数表达式 */
export const $Fn = <T extends VmFunctionLike>(name: string | null | undefined, fn: T): VmFunction<T> => {
    return VmFunction(fn, { isLib: false, name: name || VM_FUNCTION_ANONYMOUS_NAME });
};

/** 读取闭包上值 */
export const $Upvalue = <T extends VmValue>(value: T | undefined): T => {
    $AssertInit(value);
    return value;
};

/** 默认执行上下文 */
export const $GlobalFallback = (): VmContext => {
    return DefaultVmContext;
};
