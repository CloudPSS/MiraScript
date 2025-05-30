import type { VmExtern } from './extern.js';
import { isVmFunction, type VmFunction } from './function.js';
import type { VmModule } from './module.js';
import { VmWrapper } from './wrapper.js';
const { isArray } = Array;
const { getPrototypeOf } = Object;

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

/** 检查是否为 Mirascript 值 */
export function isVmValue(value: unknown): value is VmAny {
    switch (typeof value) {
        case 'string':
        case 'number':
        case 'boolean':
        case 'undefined':
            return true;
        case 'object':
            if (value == null) return true;
            if (value instanceof VmWrapper) return true;
            if (isArray(value)) {
                // VmArray 应为普通数组
                // Array.prototype
                const proto1: unknown = getPrototypeOf(value);
                if (!isArray(proto1)) return false;
                // Object.prototype
                const proto2: unknown = getPrototypeOf(proto1);
                if (proto2 == null || isArray(proto2)) return false;
                // null
                const proto3: unknown = getPrototypeOf(proto2);
                return proto3 == null;
            } else {
                // VmRecord 应为普通对象或空原型对象
                // Object.prototype
                const proto1: unknown = getPrototypeOf(value);
                if (proto1 == null) return true;
                if (isArray(proto1)) return false;
                // null
                const proto2: unknown = getPrototypeOf(proto1);
                return proto2 == null;
            }
        case 'function':
            return isVmFunction(value);
        case 'bigint':
        case 'symbol':
        default:
            return false; // Other types are not valid
    }
}

/**
 * 检查值是否为 Mirascript 数组
 */
export function isVmArray(value: VmValue): value is VmArray {
    return isArray(value);
}

/**
 * 检查值是否为 Mirascript 记录
 */
export function isVmRecord(value: VmValue): value is VmRecord {
    if (value == null || typeof value !== 'object') return false;
    if (value instanceof VmWrapper) return false;
    if (isVmArray(value)) return false;
    value satisfies VmRecord;
    return true;
}

/**
 * 检查值是否为 Mirascript 原始值
 */
export function isVmPrimitive(value: VmValue): value is VmPrimitive {
    if (value === null) return true;
    if (typeof value === 'object' || typeof value === 'function') return false;
    value satisfies VmPrimitive;
    return true;
}
