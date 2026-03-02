import type { VmExtern } from './extern.js';
import type { VmFunction } from './function.js';
import type { VmModule } from './module.js';

/** 类型名称 */
export type VmTypeName = keyof VmValueMap;

/** Mirascript 虚拟机内的未初始化变量 */
export type VmUninitialized = undefined;
/** Mirascript 原始值 */
export type VmPrimitive = null | number | string | boolean;
/**
 * Mirascript 数组
 * 数组中的 `undefined`、`null` 及 <empty slot> 均视作 `nil`
 */
export type VmArray = ReadonlyArray<VmConst | undefined>;
/**
 * Mirascript 记录
 * 仅拥有且可枚举的字符串键视作存在
 * 字段值 `undefined` 和 `null` 均视作 `nil`
 */
export type VmRecord = {
    readonly [key: string]: VmConst | undefined;
};
/** Mirascript 虚拟机内的值语义值 */
export type VmConst = VmPrimitive | VmRecord | VmArray;
/** Mirascript 虚拟机内的不可变值 */
export type VmImmutable = VmConst | VmFunction | VmModule;
/** Mirascript 虚拟机内的合法值 */
export type VmValue = VmImmutable | VmExtern;
/** Mirascript 虚拟机内的值（包括未初始化变量） */
export type VmAny = VmValue | VmUninitialized;

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

export {
    isVmAny,
    isVmArray,
    isVmArrayLikeRecord,
    isVmArrayLikeRecordByEntires,
    isVmArrayLikeRecordByKeys,
    isVmCallable,
    isVmConst,
    isVmContext,
    isVmExtern,
    isVmFunction,
    isVmImmutable,
    isVmModule,
    isVmPrimitive,
    isVmRecord,
    isVmScript,
    isVmValue,
    isVmWrapper,
    getVmFunctionInfo,
} from '../../helpers/types.js';

export { wrapToVmValue, unwrapFromVmValue } from './boundary.js';

export { type VmContext, type VmContextRecord, defineVmContextValue, createVmContext } from './context.js';
export { VmExtern } from './extern.js';
export { VmFunction, type VmFunctionInfo, type VmFunctionLike, type VmFunctionOption } from './function.js';
export { VmModule } from './module.js';
