import { VmExtern } from './extern.js';
import type { VmFunction } from './function.js';
import { VmModule } from './module.js';
import { VmWrapper } from './wrapper.js';
const { isArray } = Array;

/** Mirascript 原始值 */
export type VmPrimitive = null | string | number | boolean;
/** Mirascript 记录 */
export type VmRecord = { readonly [key: string]: VmConst };
/** Mirascript 数组 */
export type VmArray = readonly VmConst[];

/** Mirascript 虚拟机内的值语义值 */
export type VmConst = VmPrimitive | VmRecord | VmArray;

/** Mirascript 虚拟机内的不可变值 */
export type VmImmutable = VmConst | VmFunction | VmModule;

/** Mirascript 虚拟机内的合法值 */
export type VmValue = VmImmutable | VmExtern;

/** Mirascript 虚拟机内的未初始化变量 */
export type VmUninitialized = undefined;

/** Mirascript 虚拟机内的值（包括未初始化变量） */
export type VmAny = VmValue | VmUninitialized;

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

/**
 * 检查值是否为 Mirascript 数组
 */
export function isVmArray(value: VmAny): value is VmArray {
    if (!isArray(value)) return false;
    value as VmArray satisfies VmArray;
    return true;
}

/**
 * 检查值是否为 Mirascript 记录
 */
export function isVmRecord(value: VmAny): value is VmRecord {
    if (value == null || typeof value !== 'object') return false;
    if (value instanceof VmWrapper) return false;
    if (isVmArray(value)) return false;
    value satisfies VmRecord;
    return true;
}

/**
 * 检查值是否为 Mirascript 原始值
 */
export function isVmPrimitive(value: VmAny): value is VmPrimitive {
    if (value === null) return true;
    if (value === undefined || typeof value === 'object' || typeof value === 'function') return false;
    value satisfies VmPrimitive;
    return true;
}

export { VmExtern, wrapToVmValue, unwrapFromVmValue } from './extern.js';

/** 检查值是否为 Mirascript 外部值 */
export function isVmExtern(value: unknown): value is VmExtern {
    return value instanceof VmExtern;
}

export {
    VmFunction,
    isVmFunction,
    getVmFunctionInfo,
    type VmFunctionInfo,
    type VmFunctionLike,
    type VmFunctionOption,
} from './function.js';

export { VmModule } from './module.js';

/** 检查值是否为 Mirascript 模块 */
export function isVmModule(value: unknown): value is VmModule {
    return value instanceof VmModule;
}

export {
    type VmGlobal,
    type VmSharedGlobal,
    isVmGlobal,
    defineVmGlobalFunction,
    defineVmGlobalValue,
    createVmGlobal,
} from './global.js';

export { type VmScript, isVmScript } from './script.js';

export { isVmAny, isVmConst, isVmImmutable, isVmValue } from './checker.js';
