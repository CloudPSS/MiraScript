import {
    isVmArray,
    isVmRecord,
    type VmArray,
    type VmExtern,
    type VmFunction,
    type VmModule,
    type VmAny,
    type VmRecord,
} from '../vm/index.js';
import { REG_IDENTIFIER, REG_ORDINAL } from './constants.js';
import { entries, isFinite, isNaN } from '../helpers/utils.js';

const MAX_DEPTH = 100;
const REG_IDENTIFIER_FULL = new RegExp(`^${REG_IDENTIFIER.source}$`, REG_IDENTIFIER.flags);
const REG_ORDINAL_FULL = new RegExp(`^${REG_ORDINAL.source}$`, REG_ORDINAL.flags);

/**
 * 将 MiraScript 字符串序列化为 MiraScript 字面量。
 */
export function serializeString(value: string): string {
    if (!/[\p{C}'"`$\\]/u.test(value)) {
        // 不包含特殊字符
        return `'${value}'`;
    }
    let ret = "'";
    for (const char of value) {
        if (char === "'") {
            ret += String.raw`\'`;
        } else if (char === '\0') {
            ret += String.raw`\0`;
        } else if (char === '\n') {
            ret += String.raw`\n`;
        } else if (char === '\r') {
            ret += String.raw`\r`;
        } else if (char === '\t') {
            ret += String.raw`\t`;
        } else if (char === '\b') {
            ret += String.raw`\b`;
        } else if (char === '\f') {
            ret += String.raw`\f`;
        } else if (char === '\v') {
            ret += String.raw`\v`;
        } else if (char === '\\') {
            ret += String.raw`\\`;
        } else if (char === '$') {
            ret += String.raw`\$`;
        } else if (/\p{C}/u.test(char)) {
            const code = char.codePointAt(0)!;
            if (code <= 0x7f) {
                ret += String.raw`\x${code.toString(16).padStart(2, '0')}`;
            } else if (code >= 0xd800 && code <= 0xdfff) {
                // 无效的代理对
                ret += '�';
            } else {
                ret += String.raw`\u{${code.toString(16)}}`;
            }
        } else {
            ret += char; // 普通字符直接添加
        }
    }
    ret += "'";
    return ret;
}

/** 序列化属性名 */
export function serializePropName(value: string): string {
    if (REG_ORDINAL_FULL.test(value)) {
        return value; // 如果是合法的数字属性名，直接返回
    }
    if (REG_IDENTIFIER_FULL.test(value)) {
        return value; // 如果是合法的标识符，直接返回
    }
    return serializeString(value); // 否则，序列化为字符串
}

/** 序列化布尔值 */
function serializeBoolean(value: boolean): string {
    return value ? 'true' : 'false';
}

/** 序列化数字 */
function serializeNumber(value: number): string {
    if (isNaN(value)) return 'nan';
    if (!isFinite(value)) return value < 0 ? '-inf' : 'inf';
    return String(value);
}

/** 序列化数组 */
function serializeArray(value: VmArray, depth: number): string {
    if (value.length === 0) return '[]';
    let str = '[';
    for (let i = 0; i < value.length; i++) {
        if (i > 0) str += ', ';
        str += serializeImpl(value[i], depth);
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
function serializeRecord(value: VmRecord, depth: number): string {
    const customValue = customValueOf(value);
    if (customValue !== undefined) {
        return serializeImpl(customValue, depth - 1);
    }
    const e = entries(value);
    if (e.length === 0) return '()';
    if (e.length === 1) {
        const [k, v] = e[0]!;
        if (k === '0') {
            return `(${serializeImpl(v, depth)},)`; // 单个元素数组
        }
        return `(${serializePropName(k)}: ${serializeImpl(v, depth)})`;
    }

    // 根据 ES 标准，数字 key 会按顺序枚举
    const omitKey = e.length < 10 && e.every(([key], index) => key === String(index));
    let str = '(';
    for (const [key, val] of e) {
        if (str.length > 1) str += ', ';
        if (omitKey) {
            str += serializeImpl(val, depth);
        } else {
            str += `${serializePropName(key)}: ${serializeImpl(val, depth)}`;
        }
    }
    str += ')';
    return str;
}

/** 序列化 */
function serializeImpl(value: VmAny | undefined, depth: number): string {
    if (value == null || depth > MAX_DEPTH) return `nil`;
    if (typeof value == 'boolean') return serializeBoolean(value);
    if (typeof value == 'number') return serializeNumber(value);
    if (typeof value == 'string') return serializeString(value);

    if (isVmArray(value)) return serializeArray(value, depth + 1);
    if (isVmRecord(value)) return serializeRecord(value, depth + 1);
    // 不支持序列化的值
    value satisfies VmFunction | VmModule | VmExtern;
    return `nil`;
}

/**
 * 将 MiraScript 值序列化为 MiraScript 字面量字符串，非常量值将被转换为 `nil`。
 */
export function serialize(value: VmAny): string {
    return serializeImpl(value, 0);
}
