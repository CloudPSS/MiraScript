import { isVmArray, isVmRecord, type VmAny } from '../vm';
import { REG_IDENTIFIER, REG_ORDINAL } from './constants';

const MAX_DEPTH = 100;

/** 序列化字符串 */
function serializeString(value: string): string {
    if (!/[\p{C}'"`$\\]/u.test(value)) {
        // 不包含特殊字符
        return `'${value}'`;
    }
    let ret = '`';
    for (const char of value) {
        if (char === '`') {
            ret += '\\`';
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
            } else {
                ret += String.raw`\u{${code.toString(16)}}`;
            }
        } else {
            ret += char; // 普通字符直接添加
        }
    }
    ret += '`';
    return ret;
}

/** 序列化属性名 */
function serializePropName(value: string): string {
    if (REG_ORDINAL.test(value)) {
        return value; // 如果是合法的数字属性名，直接返回
    }
    if (REG_IDENTIFIER.test(value)) {
        return value; // 如果是合法的标识符，直接返回
    }
    return serializeString(value); // 否则，序列化为字符串
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
        const entries = Object.entries(value);
        if (entries.length === 0) return '()';
        if (entries.length === 1)
            return `(${serializePropName(entries[0]![0])}: ${serializeImpl(entries[0]![1], depth)},)`;
        const str = ['('];
        for (const [key, val] of Object.entries(value)) {
            if (str.length > 1) str.push(', ');
            str.push(`${serializePropName(key)}: ${serializeImpl(val, depth)}`);
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
