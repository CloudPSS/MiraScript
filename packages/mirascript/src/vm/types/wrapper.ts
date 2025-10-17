import type { TypeName, VmAny } from './index.ts';

/**
 * Mirascript 特殊值的包装器
 */
export abstract class VmWrapper<T extends object> {
    constructor(readonly value: T) {}
    /** 检查是否包含键 */
    abstract has(key: string): boolean;
    /** 获取键对应的值 */
    abstract get(key: string): VmAny;
    /** 获取键名 */
    abstract keys(): string[];
    /** 与其他值比较 */
    abstract same(other: VmAny): boolean;
    /** 获取类型 */
    abstract get type(): TypeName;
    /** 获取描述 */
    abstract get describe(): string;
    /** Convert the object to JSON */
    toJSON(): undefined {
        return undefined;
    }
    /** 转为字符串 */
    toString(): string {
        return `<${this.type} ${this.describe}>`;
    }
}
