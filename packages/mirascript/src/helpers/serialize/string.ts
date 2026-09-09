import type { SerializeOptions } from './interface.js';

/**
 * 将 MiraScript 转义字符串序列化为 MiraScript 字面量。
 */
function serializeStringEscaped(escaped: string, options: Readonly<SerializeOptions>): string {
    return options.serializeStringEscape('\\' + escaped, options);
}

/**
 * 将 MiraScript 普通字符串序列化为 MiraScript 字面量。
 */
function serializeStringContent(value: string, options: Readonly<SerializeOptions>): string {
    return options.serializeStringContent(value, options);
}

const STRING_QUOTE = `'`;
const STRING_REG = new RegExp(String.raw`[${STRING_QUOTE}\\\$\p{C}\u2028\u2029]`, 'gu');
const STRING_MARK = /^[\p{M}]$/u;

/**
 * 序列化为特殊字符
 */
function serializeSpecialChar(char: string, options: Readonly<SerializeOptions>): string {
    if (char === STRING_QUOTE) {
        return serializeStringEscaped(STRING_QUOTE, options);
    } else if (char === '\0') {
        return serializeStringEscaped(`0`, options);
    } else if (char === '\n') {
        return serializeStringEscaped(`n`, options);
    } else if (char === '\r') {
        return serializeStringEscaped(`r`, options);
    } else if (char === '\t') {
        return serializeStringEscaped(`t`, options);
    } else if (char === '\b') {
        return serializeStringEscaped(`b`, options);
    } else if (char === '\f') {
        return serializeStringEscaped(`f`, options);
    } else if (char === '\v') {
        return serializeStringEscaped(`v`, options);
    } else if (char === '\\') {
        return serializeStringEscaped(`\\`, options);
    } else if (char === '$') {
        return serializeStringEscaped(`$`, options);
    } else {
        const code = char.codePointAt(0)!;
        if (code <= 0x7f) {
            return serializeStringEscaped(`x${code.toString(16).padStart(2, '0')}`, options);
        } else if (code >= 0xd800 && code <= 0xdfff) {
            // 无效的代理对
            return serializeStringContent('�', options);
        } else {
            return serializeStringEscaped(`u{${code.toString(16)}}`, options);
        }
    }
}

/**
 * 将 MiraScript 字符串序列化为 MiraScript 字面量。
 */
export function serializeStringImpl(value: string, options: Readonly<SerializeOptions>): string {
    const oq = options.serializeStringQuote(STRING_QUOTE, true, options);
    const cq = options.serializeStringQuote(STRING_QUOTE, false, options);
    if (value.length === 0) {
        return oq + cq;
    }

    let ret = oq;
    let lastIndex = 0;

    // 当开头字符是 \p{M} 时，使用转义序列化
    while (value.length > lastIndex) {
        const cp = value.codePointAt(lastIndex)!;
        const ch = String.fromCodePoint(cp);
        if (!STRING_MARK.test(ch)) break;
        ret += serializeStringEscaped(`u{${cp.toString(16)}}`, options);
        lastIndex += ch.length;
    }

    // 序列化字符串内容，遇到特殊字符时使用转义序列化
    STRING_REG.lastIndex = lastIndex;
    let match: RegExpExecArray | null;
    while ((match = STRING_REG.exec(value)) !== null) {
        ret += serializeStringContent(value.slice(lastIndex, match.index), options);
        lastIndex = STRING_REG.lastIndex;
        ret += serializeSpecialChar(match[0], options);
    }
    ret += serializeStringContent(value.slice(lastIndex), options);
    ret += cq;
    return ret;
}
