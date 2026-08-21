import type { VmPrimitive } from '../../index.js';

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

/** 创建数组 */
export function createArray<T>(length: number, fn: (index: number) => T): T[] {
    // micro bench shows that this is faster than Array.from
    const result: T[] = [];
    for (let i = 0; i < length; i++) {
        result.push(fn(i));
    }
    return result;
}
