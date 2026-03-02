import { kVmWrapper } from '../../helpers/constants.js';
import type { VmTypeName, VmAny } from './index.js';

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
    abstract get type(): VmTypeName;
    /** 获取当前对象的描述 */
    abstract get tag(): string;
    /** 描述键对应的值 */
    describe(key: string): string | undefined {
        return undefined;
    }
    /** 转为字符串 */
    toString(useBraces: boolean): string {
        const { type, tag } = this;
        if (!tag) return `<${type}>`;
        return `<${type} ${tag}>`;
    }
}

Object.defineProperty(VmWrapper.prototype, kVmWrapper, { value: true });
