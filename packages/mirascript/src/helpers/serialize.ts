import type { VmArray, VmExtern, VmFunction, VmModule, VmAny, VmRecord } from '../vm/index.js';
import { REG_IDENTIFIER_FULL, REG_ORDINAL_FULL } from './constants.js';
import { entries, hasOwn, isFinite, isNaN } from '../helpers/utils.js';
import {
    getVmFunctionInfo,
    isVmArray,
    isVmArrayLikeRecordByEntires,
    isVmExtern,
    isVmFunction,
    isVmModule,
    isVmRecord,
} from './types.js';

/** 序列化设置 */
export interface SerializeOptions {
    /** 最大递归深度，超过该深度的值将被序列化为 `nil`，默认值为 128 */
    maxDepth: number;
    /** 序列化 nil 值 */
    serializeNil: (options: SerializeOptions) => string;
    /** 序列化布尔值 */
    serializeBoolean: (value: boolean, options: SerializeOptions) => string;
    /** 序列化数字 */
    serializeNumber: (value: number, options: SerializeOptions) => string;
    /** 序列化字符串 */
    serializeString: (value: string, options: SerializeOptions) => string;
    /** 序列化字符串引号 */
    serializeStringQuote: (value: string, open: boolean, options: SerializeOptions) => string;
    /** 序列化字符串转义序列 */
    serializeStringEscape: (value: string, options: SerializeOptions) => string;
    /** 序列化字符串常规内容 */
    serializeStringContent: (value: string, options: SerializeOptions) => string;
    /** 序列化函数 */
    serializeFunction: (value: VmFunction, options: SerializeOptions) => string;
    /** 序列化数组 */
    serializeArray: (value: VmArray, depth: number, options: SerializeOptions) => string;
    /** 序列化记录 */
    serializeRecord: (value: VmRecord, depth: number, options: SerializeOptions) => string;
    /** 序列化属性名 */
    serializePropName: (value: number | string, options: SerializeOptions) => string;
    /** 序列化模块 */
    serializeModule: (value: VmModule, depth: number, options: SerializeOptions) => string;
    /** 序列化外部值 */
    serializeExtern: (value: VmExtern, depth: number, options: SerializeOptions) => string;
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

/**
 * 将 MiraScript 字符串序列化为 MiraScript 字面量。
 */
function serializeStringImpl(value: string, options: Readonly<SerializeOptions>): string {
    if (value.length === 0) {
        const oq = options.serializeStringQuote(`'`, true, options);
        const cq = options.serializeStringQuote(`'`, false, options);
        return oq + cq;
    }
    if (value.length === 1 && /[\p{M}\p{C}]/u.test(value)) {
        const oq = options.serializeStringQuote(`'`, true, options);
        const cq = options.serializeStringQuote(`'`, false, options);
        const c = options.serializeStringEscape(String.raw`\u{${value.codePointAt(0)!.toString(16)}}`, options);
        return oq + c + cq;
    }
    if (!/[\\'"`$\p{C}\u2028\u2029]/u.test(value)) {
        // 不包含特殊字符
        const oq = options.serializeStringQuote(`'`, true, options);
        const cq = options.serializeStringQuote(`'`, false, options);
        const c = options.serializeStringContent(value, options);
        return oq + c + cq;
    }
    let ret = options.serializeStringQuote(`'`, true, options);
    for (const char of value) {
        if (char === "'") {
            ret += options.serializeStringEscape(String.raw`\'`, options);
        } else if (char === '\0') {
            ret += options.serializeStringEscape(String.raw`\0`, options);
        } else if (char === '\n') {
            ret += options.serializeStringEscape(String.raw`\n`, options);
        } else if (char === '\r') {
            ret += options.serializeStringEscape(String.raw`\r`, options);
        } else if (char === '\t') {
            ret += options.serializeStringEscape(String.raw`\t`, options);
        } else if (char === '\b') {
            ret += options.serializeStringEscape(String.raw`\b`, options);
        } else if (char === '\f') {
            ret += options.serializeStringEscape(String.raw`\f`, options);
        } else if (char === '\v') {
            ret += options.serializeStringEscape(String.raw`\v`, options);
        } else if (char === '\\') {
            ret += options.serializeStringEscape(String.raw`\\`, options);
        } else if (char === '$') {
            ret += options.serializeStringEscape(String.raw`\$`, options);
        } else if (/[\p{C}\u2028\u2029]/u.test(char)) {
            const code = char.codePointAt(0)!;
            if (code <= 0x7f) {
                ret += options.serializeStringEscape(String.raw`\x${code.toString(16).padStart(2, '0')}`, options);
            } else if (code >= 0xd800 && code <= 0xdfff) {
                // 无效的代理对
                ret += options.serializeStringContent('�', options);
            } else {
                ret += options.serializeStringEscape(String.raw`\u{${code.toString(16)}}`, options);
            }
        } else {
            ret += options.serializeStringContent(char, options); // 普通字符直接添加
        }
    }
    ret += options.serializeStringQuote(`'`, false, options);
    return ret;
}

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

/** 序列化 nil 值 */
export function serializeNil(): string {
    return 'nil';
}

/** 序列化布尔值 */
export function serializeBoolean(value: boolean): string {
    return value ? 'true' : 'false';
}

/** 序列化数字 */
export function serializeNumber(value: number): string {
    if (isNaN(value)) return 'nan';
    if (!isFinite(value)) return value < 0 ? '-inf' : 'inf';
    if (value === 0) {
        if (1 / value < 0) return '-0';
        return '0';
    }
    return String(value);
}

/** 序列化数组 */
export function serializeArray(value: VmArray, depth: number, options: Readonly<SerializeOptions>): string {
    if (depth > options.maxDepth) return `[]`;
    if (value.length === 0) return '[]';
    let str = '[';
    for (let i = 0; i < value.length; i++) {
        if (i > 0) str += ', ';
        str += serializeImpl(value[i], depth, options);
    }
    str += ']';
    return str;
}

// eslint-disable-next-line @typescript-eslint/unbound-method
const { valueOf } = Object.prototype;
/**
 * 如果值有自定义的 valueOf 方法，调用它并返回结果，否则返回 undefined。
 */
function customValueOf(value: VmRecord): VmAny | undefined {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const thisValueOf = value.valueOf;
    if (typeof thisValueOf != 'function' || thisValueOf === valueOf) {
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

    const omitKey = isVmArrayLikeRecordByEntires(e);
    let str = '(';
    for (const [key, val] of e) {
        if (str.length > 1) str += ', ';
        if (omitKey) {
            str += serializeImpl(val, depth, options);
        } else {
            str += `${serializeRecordKeyOpt(key, options)}: ${serializeImpl(val, depth, options)}`;
        }
    }
    str += ')';
    return str;
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

/** 将 MiraScript 函数转化为 MiraScript 字符串 */
export function displayFunction(value: VmFunction): string {
    try {
        const name = getVmFunctionInfo(value)?.fullName;
        return name ? `<function ${name}>` : `<function>`;
        /* c8 ignore next 3 */
    } catch {
        return `<function>`;
    }
}
/** 将 MiraScript 模块转化为 MiraScript 字符串 */
export function displayModule(value: VmModule): string {
    try {
        return value.toString(true);
        /* c8 ignore next 3 */
    } catch {
        return `<module>`;
    }
}
/** 将 MiraScript 外部值转化为 MiraScript 字符串 */
export function displayExtern(value: VmExtern): string {
    try {
        const tag = `<extern ${value.tag}>`;
        const rep = value.toString(true);
        if (rep === tag || rep.length > 50) {
            return tag;
        }
        return `${tag} ${rep}`;
        /* c8 ignore next 3 */
    } catch {
        return `<extern>`;
    }
}
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
/**
 * 将 MiraScript 值转化为 MiraScript 字符串。
 */
export function display(value: VmAny, options?: Partial<SerializeOptions>): string {
    const opt = mergeOptions(DISPLAY_OPTIONS, options);
    return serializeImpl(value, 0, opt);
}
