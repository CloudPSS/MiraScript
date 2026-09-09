import { REG_IDENTIFIER_FULL, REG_ORDINAL_FULL } from '@mirascript/constants';
import type { VmArray, VmAny, VmRecord } from '../../vm/index.js';
import { entries, hasOwn } from '../utils.js';
import {
    isVmArray,
    isVmArrayLikeRecordByEntries,
    isVmExtern,
    isVmFunction,
    isVmModule,
    isVmRecord,
} from '../types/index.js';
import type { SerializeOptions } from './interface.js';
import { serializeNil, serializeBoolean, serializeNumber } from './simple.js';
import { serializeStringImpl } from './string.js';
import { displayExtern, displayFunction, displayModule } from './display.js';

export type { SerializeOptions };
export { serializeNil, serializeBoolean, serializeNumber };

/**
 * 将 MiraScript 字符串序列化为 MiraScript 字面量。
 */
export function serializeString(value: string, options?: Partial<SerializeOptions>): string {
    return serializeStringImpl(value, getSerializeOptions(options));
}

/** 使用默认选项序列化属性名 */
function serializeRecordKeyDefault(key: string): string {
    if (REG_ORDINAL_FULL.test(key) || REG_IDENTIFIER_FULL.test(key)) {
        return key;
    }
    return serializeStringImpl(key, DEFAULT_OPTIONS);
}

/** 序列化属性名 */
function serializeRecordKeyOpt(value: string, options: Readonly<SerializeOptions>): string {
    if (isDefaultOptions(options)) {
        return serializeRecordKeyDefault(value);
    }
    if (REG_ORDINAL_FULL.test(value)) {
        // 合法的数字属性名
        return options.serializePropName(Number(value), options);
    }
    if (REG_IDENTIFIER_FULL.test(value)) {
        // 合法的标识符
        return options.serializePropName(value, options);
    }
    // 否则，序列化为字符串
    return options.serializeString(value, options);
}

/** 序列化属性名 */
export function serializeRecordKey(key: string, options?: Partial<SerializeOptions>): string {
    if (isDefaultOptions(options)) {
        return serializeRecordKeyDefault(key);
    }
    return serializeRecordKeyOpt(key, getSerializeOptions(options));
}

/** 序列化数组 */
export function serializeArray(value: VmArray, depth: number, options: Readonly<SerializeOptions>): string {
    if (depth > options.maxDepth) return `[]`;
    if (value.length === 0) return '[]';
    let result = '[';
    for (let i = 0; i < value.length; i++) {
        if (i > 0) result += ', ';
        result += serializeImpl(value[i], depth, options);
    }
    result += ']';
    return result;
}

// eslint-disable-next-line @typescript-eslint/unbound-method
const { valueOf } = Object.prototype;
/**
 * 如果值有自定义的 valueOf 方法，调用它并返回结果，否则返回 undefined。
 */
function customValueOf(value: VmRecord): VmAny | undefined {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const thisValueOf = value.valueOf;
    if (typeof thisValueOf !== 'function' || thisValueOf === valueOf) {
        return undefined;
    }
    const customValue = thisValueOf.call(value) as VmAny | undefined;
    if (customValue === value) return undefined;
    return customValue;
}

/** 序列化记录 */
export function serializeRecord(value: VmRecord, depth: number, options: Readonly<SerializeOptions>): string {
    const customValue = customValueOf(value);
    if (customValue !== undefined) {
        return serializeImpl(customValue, depth - 1, options);
    }
    if (depth > options.maxDepth) return `()`;
    const e = entries(value);
    if (e.length === 0) return '()';
    if (e.length === 1) {
        const [k, v] = e[0]!;
        if (k === '0') {
            return `(${serializeImpl(v, depth, options)},)`; // 单个元素数组
        }
        return `(${serializeRecordKeyOpt(k, options)}: ${serializeImpl(v, depth, options)})`;
    }

    const omitKey = isVmArrayLikeRecordByEntries(e);
    let result = '(';
    for (const [key, val] of e) {
        if (result.length > 1) result += ', ';
        if (omitKey) {
            result += serializeImpl(val, depth, options);
        } else {
            result += `${serializeRecordKeyOpt(key, options)}: ${serializeImpl(val, depth, options)}`;
        }
    }
    result += ')';
    return result;
}

/** 序列化 */
function serializeImpl(value: VmAny | undefined, depth: number, options: Readonly<SerializeOptions>): string {
    if (value == null) {
        return options.serializeNil(options);
    }
    if (typeof value == 'boolean') {
        return options.serializeBoolean(value, options);
    }
    if (typeof value == 'number') {
        return options.serializeNumber(value, options);
    }
    if (typeof value == 'string') {
        return options.serializeString(value, options);
    }
    if (isVmFunction(value)) {
        return options.serializeFunction(value, options);
    }
    if (isVmModule(value)) {
        return options.serializeModule(value, depth + 1, options);
    }
    if (isVmExtern(value)) {
        return options.serializeExtern(value, depth + 1, options);
    }
    if (isVmArray(value)) {
        return options.serializeArray(value, depth + 1, options);
    }
    if (isVmRecord(value)) {
        return options.serializeRecord(value, depth + 1, options);
    }
    // 不支持序列化的值
    value satisfies never;
    return options.serializeNil(options);
}

/**
 * 将 MiraScript 值序列化为 MiraScript 字面量字符串，非常量值默认转换为 `nil`。
 */
export function serialize(value: VmAny, options?: Partial<SerializeOptions>): string {
    return serializeImpl(value, 0, getSerializeOptions(options));
}

/**
 * 将 MiraScript 值转化为 MiraScript 字符串。
 */
export function display(value: VmAny, options?: Partial<SerializeOptions>): string {
    const opt = mergeOptions(DISPLAY_OPTIONS, options);
    return serializeImpl(value, 0, opt);
}

/** 是否为默认选项 */
function isDefaultOptions(options: Partial<SerializeOptions> | undefined): boolean {
    return options == null || options === DEFAULT_OPTIONS;
}

/** 合并选项 */
function mergeOptions(
    base: Readonly<SerializeOptions>,
    options: Partial<SerializeOptions> | null | undefined,
): Readonly<SerializeOptions> {
    if (options == null) return base;
    let opt: SerializeOptions | null = null;
    for (const key in options) {
        if (!hasOwn(options, key) || !hasOwn(base, key)) continue;
        const el = options[key as keyof SerializeOptions];
        if (el == null) continue;
        opt ??= { ...base };
        opt[key as keyof SerializeOptions] = el as never;
    }
    return opt ? Object.freeze(opt) : base;
}

/** 获取选项 */
function getSerializeOptions(options: Partial<SerializeOptions> | undefined): Readonly<SerializeOptions> {
    if (isDefaultOptions(options)) return DEFAULT_OPTIONS;
    return mergeOptions(DEFAULT_OPTIONS, options);
}

const DEFAULT_OPTIONS = Object.freeze({
    maxDepth: 128,
    serializeNil,
    serializeBoolean,
    serializeNumber,
    serializeString: serializeStringImpl,
    serializeStringQuote: (value) => value,
    serializeStringEscape: (value) => value,
    serializeStringContent: (value) => value,
    serializeArray,
    serializeRecord,
    serializePropName: String,
    serializeFunction: serializeNil,
    serializeModule: serializeNil,
    serializeExtern: serializeNil,
} satisfies SerializeOptions);

const DISPLAY_OPTIONS = Object.freeze({
    maxDepth: 3,
    serializeNil,
    serializeBoolean,
    serializeNumber,
    serializeString: serializeStringImpl,
    serializeStringQuote: (value) => value,
    serializeStringEscape: (value) => value,
    serializeStringContent: (value) => value,
    serializeArray,
    serializeRecord,
    serializePropName: String,
    serializeFunction: displayFunction,
    serializeModule: displayModule,
    serializeExtern: displayExtern,
} satisfies SerializeOptions);
