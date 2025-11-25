import type { VmPrimitive } from '../../vm/index.js';

/** 将值转为 JS 字面量 */
export function toJsLiteral(value: VmPrimitive | undefined): string {
    /* c8 ignore next 2 */
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value == 'string') {
        return JSON.stringify(value);
    }
    // JSON 无法处理 NaN 等特殊数字
    if (value === 0) {
        if (1 / value === -Infinity) return '-0';
        return '0';
    }
    return String(value satisfies number | boolean);
}

/** 解析常量 */
export function readConst(reader: DataView, offset: number): [value: VmPrimitive, consumed: number] {
    const type = reader.getUint8(offset);
    switch (type) {
        /* c8 ignore next 2 */
        case 0:
            return [null, 1];
        case 1:
            return [true, 1];
        case 2:
            return [false, 1];
        case 3: {
            const ordinal = reader.getInt32(offset + 1, true);
            return [ordinal, 5];
        }
        case 4: {
            const num = reader.getFloat64(offset + 1, true);
            return [num, 9];
        }
        case 5: {
            const len = reader.getUint32(offset + 1, true);
            const str = new TextDecoder().decode(new Uint8Array(reader.buffer, reader.byteOffset + offset + 5, len));
            return [str, 5 + len];
        }
        /* c8 ignore next 2 */
        default:
            throw new Error(`Unknown constant type: ${type}`);
    }
}
