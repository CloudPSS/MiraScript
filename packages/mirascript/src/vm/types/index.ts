import { isArray } from '../../helpers/utils.js';
import { VmExtern } from './extern.js';
import { isVmFunction, type VmFunction } from './function.js';
import { VmModule } from './module.js';
import { VmWrapper } from './wrapper.js';

/** Mirascript 原始值 */
export type VmPrimitive = null | string | number | boolean;
/**
 * Mirascript 记录
 * 仅拥有且可枚举的字符串键视作存在
 * 字段值 `undefined` 和 `null` 均视作 `nil`
 */
export type VmRecord = {
    readonly [key: string]: VmConst | undefined;
};
/**
 * Mirascript 数组
 * 数组中的 `undefined`、`null` 及 <empty slot> 均视作 `nil`
 */
export type VmArray = ReadonlyArray<VmConst | undefined>;

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
export function isVmPrimitive(value: unknown): value is VmPrimitive {
    if (value === null || typeof value == 'number' || typeof value == 'string' || typeof value == 'boolean') {
        value as VmPrimitive satisfies typeof value;
        value satisfies VmPrimitive;
        return true;
    }
    return false;
}

export { VmExtern, wrapToVmValue, unwrapFromVmValue } from './extern.js';

/** 检查值是否为 Mirascript 外部值 */
export function isVmExtern(value: unknown): value is VmExtern {
    return value instanceof VmExtern;
}

/** 检查值是否为 Mirascript 可调用值 */
// eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
export function isVmCallable(value: unknown): value is VmFunction | VmExtern<Function> {
    return isVmFunction(value) || (isVmExtern(value) && typeof value.value == 'function');
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

export { type VmContext, type VmSharedContext, isVmContext, defineVmContextValue, createVmContext } from './context.js';

export { type VmScript, isVmScript } from './script.js';

export { isVmAny, isVmConst, isVmImmutable, isVmValue } from './checker.js';

export const VM_ARRAY_MAX_LENGTH = 2 ** 31 - 1;
