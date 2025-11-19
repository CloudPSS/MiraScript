/** Mirascript 原始值 */
export type VmPrimitive = null | string | number | boolean;

/**
 * 检查值是否为 Mirascript 原始值
 */
export function isVmPrimitive(value: unknown): value is VmPrimitive {
    if (value === null || typeof value == 'number' || typeof value == 'string' || typeof value == 'boolean') {
        value as VmPrimitive satisfies typeof value;
        value satisfies VmPrimitive;
        return true;
    }
    return false;
}
