/* eslint-disable no-useless-assignment */
import type { VmPrimitive } from '../../vm/index.js';
const { stringify } = JSON;
/** 将值转为 JS 字面量 */
export function toJsLiteral(value: VmPrimitive | undefined): string {
    /* c8 ignore next 2 */
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value == 'string') {
        return stringify(value);
    }
    // JSON 无法处理 NaN 等特殊数字
    if (value === 0) {
        if (1 / value === -Infinity) return '-0';
        return '0';
    }
    return String(value satisfies number | boolean);
}

const DECODER = new TextDecoder();
const { fromCharCode } = String;
/** 解码 Ascii */
function shortStringInJS(buf: Uint8Array, begin: number, length: number): string | undefined {
    if (length < 4) {
        if (length < 2) {
            if (length === 0) return '';
            const a = buf[begin]!;
            if ((a & 0x80) > 0) {
                return;
            }
            return fromCharCode(a);
        }
        const a = buf[begin++]!;
        const b = buf[begin++]!;
        if ((a & 0x80) > 0 || (b & 0x80) > 0) {
            return;
        }
        if (length < 3) return fromCharCode(a, b);
        const c = buf[begin++]!;
        if ((c & 0x80) > 0) {
            return;
        }
        return fromCharCode(a, b, c);
    }
    const a = buf[begin++]!;
    const b = buf[begin++]!;
    const c = buf[begin++]!;
    const d = buf[begin++]!;
    if ((a & 0x80) > 0 || (b & 0x80) > 0 || (c & 0x80) > 0 || (d & 0x80) > 0) {
        return;
    }
    if (length < 6) {
        if (length === 4) return fromCharCode(a, b, c, d);
        const e = buf[begin++]!;
        if ((e & 0x80) > 0) {
            return;
        }
        return fromCharCode(a, b, c, d, e);
    }
    const e = buf[begin++]!;
    const f = buf[begin++]!;
    if ((e & 0x80) > 0 || (f & 0x80) > 0) {
        return;
    }
    if (length < 8) {
        if (length < 7) return fromCharCode(a, b, c, d, e, f);
        const g = buf[begin++]!;
        if ((g & 0x80) > 0) {
            return;
        }
        return fromCharCode(a, b, c, d, e, f, g);
    }
    const g = buf[begin++]!;
    const h = buf[begin++]!;
    if ((g & 0x80) > 0 || (h & 0x80) > 0) {
        return;
    }
    if (length < 10) {
        if (length === 8) return fromCharCode(a, b, c, d, e, f, g, h);
        const i = buf[begin++]!;
        if ((i & 0x80) > 0) {
            return;
        }
        return fromCharCode(a, b, c, d, e, f, g, h, i);
    }
    const i = buf[begin++]!;
    const j = buf[begin++]!;
    if ((i & 0x80) > 0 || (j & 0x80) > 0) {
        return;
    }
    if (length < 12) {
        if (length < 11) return fromCharCode(a, b, c, d, e, f, g, h, i, j);
        const k = buf[begin++]!;
        if ((k & 0x80) > 0) {
            return;
        }
        return fromCharCode(a, b, c, d, e, f, g, h, i, j, k);
    }
    const k = buf[begin++]!;
    const l = buf[begin++]!;
    if ((k & 0x80) > 0 || (l & 0x80) > 0) {
        return;
    }
    if (length < 14) {
        if (length === 12) return fromCharCode(a, b, c, d, e, f, g, h, i, j, k, l);
        const m = buf[begin++]!;
        if ((m & 0x80) > 0) {
            begin -= 13;
            return;
        }
        return fromCharCode(a, b, c, d, e, f, g, h, i, j, k, l, m);
    }
    const m = buf[begin++]!;
    const n = buf[begin++]!;
    if ((m & 0x80) > 0 || (n & 0x80) > 0) {
        return;
    }
    if (length < 15) return fromCharCode(a, b, c, d, e, f, g, h, i, j, k, l, m, n);
    const o = buf[begin++]!;
    if ((o & 0x80) > 0) {
        return;
    }
    return fromCharCode(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o);
}

/** 解码字符串 */
function decodeStr(buffer: Uint8Array, offset: number, length: number): string {
    // Fast path for short ascii strings
    if (length < 16) {
        const shortStr = shortStringInJS(buffer, offset, length);
        if (shortStr !== undefined) {
            return shortStr;
        }
    }
    return DECODER.decode(buffer.subarray(offset, offset + length));
}

/** 读取常量表 */
export function readConsts(constChunk: Uint8Array): VmPrimitive[] {
    const reader = new DataView(constChunk.buffer, constChunk.byteOffset, constChunk.byteLength);
    const consts: VmPrimitive[] = [];
    let offset = 0;
    const length = reader.byteLength;
    while (offset < length) {
        const type = reader.getUint8(offset);
        switch (type) {
            /* c8 ignore next 2 */
            case 0:
                consts.push(null);
                offset += 1;
                break;
            case 1:
                consts.push(true);
                offset += 1;
                break;
            case 2:
                consts.push(false);
                offset += 1;
                break;
            case 3: {
                const ordinal = reader.getInt32(offset + 1, true);
                consts.push(ordinal);
                offset += 5;
                break;
            }
            case 4: {
                const num = reader.getFloat64(offset + 1, true);
                consts.push(num);
                offset += 9;
                break;
            }
            case 5: {
                const len = reader.getUint32(offset + 1, true);
                const str = decodeStr(constChunk, offset + 5, len);
                consts.push(str);
                offset += 5 + len;
                break;
            }
            /* c8 ignore next 2 */
            default:
                throw new Error(`Unknown constant type: ${type}`);
        }
    }
    return consts;
}
