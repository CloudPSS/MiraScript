import type { VmAny } from '../../vm/index.js';
import { VmError } from '../error.js';
import { display } from '../serialize.js';
import { isNaN, NegativeInfinity, NotNumber, PositiveInfinity } from '../utils.js';

/** 将字符串转换为数字 */
function parseNumericLiteral(value: string): number | null {
    if (value === '') return null;
    const ch0 = value.charAt(0);
    if (ch0 < '0' || ch0 > '9') {
        return null;
    }
    // 二进制
    if (/^0[bB][01]+$/.test(value)) {
        const num = Number.parseInt(value.slice(2), 2);
        return isNaN(num) ? null : num;
    }
    // 八进制
    if (/^0[oO][0-7]+$/.test(value)) {
        const num = Number.parseInt(value.slice(2), 8);
        return isNaN(num) ? null : num;
    }
    // 十六进制
    if (/^0[xX][0-9a-fA-F]+$/.test(value)) {
        const num = Number.parseInt(value.slice(2), 16);
        return isNaN(num) ? null : num;
    }
    // 十进制/科学计数法
    if (/^[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?$/.test(value)) {
        const num = Number.parseFloat(value);
        return isNaN(num) ? null : num;
    }
    return null;
}

/** 将字符串转换为数字 */
function stringToNumber(value: string): number | null {
    value = value.trim();
    if (value === '') return null;
    if (value === 'inf' || value === '+inf' || value === 'Infinity' || value === '+Infinity') {
        return PositiveInfinity;
    }
    if (value === '-inf' || value === '-Infinity') {
        return NegativeInfinity;
    }
    if (value === 'nan' || value === 'NaN') {
        return NotNumber;
    }
    if (value.startsWith('-')) {
        const num = parseNumericLiteral(value.slice(1));
        if (num !== null) return -num;
        return null;
    } else if (value.startsWith('+')) {
        return parseNumericLiteral(value.slice(1));
    } else {
        return parseNumericLiteral(value);
    }
}

/** 转换为 number */
export function toNumber<F = undefined>(value: VmAny, fallback?: F): number | Exclude<F, undefined> {
    if (typeof value === 'number') return value;
    if (typeof value == 'boolean') return value ? 1 : 0;
    if (typeof value == 'string') {
        const num = stringToNumber(value);
        if (num !== null) return num;
    }
    if (fallback === undefined) {
        throw new VmError(`Failed to convert value to number: ${display(value)}`, NotNumber);
    }
    return fallback as Exclude<F, undefined>;
}
