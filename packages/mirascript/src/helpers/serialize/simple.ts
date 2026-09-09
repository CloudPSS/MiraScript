import { isNaN } from '../utils.js';

/** 序列化 nil 值 */
export function serializeNil(): string {
    return 'nil';
}

/** 序列化布尔值 */
export function serializeBoolean(value: boolean): string {
    return value ? 'true' : 'false';
}

/** 转换为 string */
export function numberToString(value: number, minusZero: boolean): string {
    if (isNaN(value)) return 'nan';
    if (value === Infinity) return 'inf';
    if (value === -Infinity) return '-inf';
    if (minusZero && value === 0 && 1 / value < 0) return '-0';
    return String(value);
}

/** 序列化数字 */
export function serializeNumber(value: number): string {
    return numberToString(value, true);
}
