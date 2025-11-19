import type { VmArray } from './array.js';
import type { VmExtern } from './extern.js';
import type { VmFunction } from './function.js';
import type { VmModule } from './module.js';
import type { VmRecord } from './record.js';

/** 类型名称 */
export type TypeName = keyof VmValueMap;

/** 类型名称映射 */
export interface VmValueMap {
    /** 空值 */
    nil: null;
    /** 字符串 */
    string: string;
    /** 数字 */
    number: number;
    /** 布尔值 */
    boolean: boolean;
    /** Mirascript 记录 */
    record: VmRecord;
    /** Mirascript 数组 */
    array: VmArray;
    /** Mirascript 函数 */
    function: VmFunction;
    /** 外部调用对象 */
    extern: VmExtern;
    /** Mirascript 模块 */
    module: VmModule;
}

export { wrapToVmValue, unwrapFromVmValue, toVmFunctionProxy, fromVmFunctionProxy } from './boundary.js';
export { type VmContext, type VmSharedContext, isVmContext, defineVmContextValue, createVmContext } from './context.js';
export { type VmScript, isVmScript } from './script.js';
export { isVmCallable } from './callable.js';

export { type VmArray, VM_ARRAY_MAX_LENGTH, isVmArray } from './array.js';
export {
    type VmRecord,
    isVmRecord,
    isVmArrayLikeRecord,
    isVmArrayLikeRecordByEntires,
    isVmArrayLikeRecordByKeys,
} from './record.js';
export { type VmPrimitive, isVmPrimitive } from './primitive.js';
export { type VmConst, isVmConst } from './const.js';
export { type VmImmutable, isVmImmutable } from './immutable.js';
export { VmExtern, isVmExtern } from './extern.js';
export {
    VmFunction,
    isVmFunction,
    getVmFunctionInfo,
    type VmFunctionInfo,
    type VmFunctionLike,
    type VmFunctionOption,
} from './function.js';
export { VmModule, isVmModule } from './module.js';
export { isVmWrapper } from './wrapper.js';
export { type VmValue, isVmValue } from './value.js';
export { type VmAny, type VmUninitialized, isVmAny } from './any.js';
