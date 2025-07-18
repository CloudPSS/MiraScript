import { isVmArray, isVmRecord, type VmAny, type VmRecord } from '../vm/index.js';
import { REG_IDENTIFIER, REG_ORDINAL } from './constants.js';

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

/** 序列化 */
function serializeImpl(value: VmAny | undefined, depth: number): string {
    if (value == null || depth > MAX_DEPTH) return `nil`;
    if (typeof value == 'boolean') return value ? 'true' : 'false';
    if (typeof value == 'number') {
        if (Number.isNaN(value)) return 'nan';
        if (!Number.isFinite(value)) return value < 0 ? '-inf' : 'inf';
        return String(value);
    }
    if (typeof value == 'string') return serializeString(value);

    depth += 1;
    if (isVmArray(value)) {
        if (value.length === 0) return '[]';
        const str = ['['];
        for (let i = 0; i < value.length; i++) {
            if (i > 0) str.push(', ');
            str.push(serializeImpl(value[i], depth));
        }
        str.push(']');
        return str.join('');
    }
    if (isVmRecord(value)) {
        const customValue = customValueOf(value);
        if (customValue !== undefined) {
            return serializeImpl(customValue, depth - 1);
        }
        const entries = Object.entries(value);
        if (entries.length === 0) return '()';
        if (entries.length === 1) {
            const [k, v] = entries[0]!;
            if (k === '0') {
                return `(${serializeImpl(v, depth)},)`; // 单个元素数组
            }
            return `(${serializePropName(k)}: ${serializeImpl(v, depth)})`;
        }

        // 根据 ES 标准，数字 key 会按顺序枚举
        const omitKey = entries.length < 10 && entries.every(([key], index) => key === String(index));
        const str = ['('];
        for (const [key, val] of entries) {
            if (str.length > 1) str.push(', ');
            if (omitKey) {
                str.push(serializeImpl(val, depth));
            } else {
                str.push(serializePropName(key), ': ', serializeImpl(val, depth));
            }
        }
        str.push(')');
        return str.join('');
    }
    return `nil`;
}

/**
 * 将 MiraScript 值序列化为 MiraScript 字面量字符串，非常量值将被转换为 `nil`。
 */
export function serialize(value: VmAny): string {
    return serializeImpl(value, 0);
}
