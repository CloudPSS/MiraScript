import type { VmArray, VmExtern, VmFunction, VmModule, VmRecord } from '../../vm/index.js';

/** 序列化设置 */
export interface SerializeOptions {
    /** 最大递归深度，超过该深度的值将被序列化为 `nil`，默认值为 128 */
    maxDepth: number;
    /** 序列化 nil 值 */
    serializeNil: (options: SerializeOptions) => string;
    /** 序列化布尔值 */
    serializeBoolean: (value: boolean, options: SerializeOptions) => string;
    /** 序列化数字 */
    serializeNumber: (value: number, options: SerializeOptions) => string;
    /** 序列化字符串 */
    serializeString: (value: string, options: SerializeOptions) => string;
    /** 序列化字符串引号 */
    serializeStringQuote: (value: string, open: boolean, options: SerializeOptions) => string;
    /** 序列化字符串转义序列 */
    serializeStringEscape: (value: string, options: SerializeOptions) => string;
    /** 序列化字符串常规内容 */
    serializeStringContent: (value: string, options: SerializeOptions) => string;
    /** 序列化数组 */
    serializeArray: (value: VmArray, depth: number, options: SerializeOptions) => string;
    /** 序列化记录 */
    serializeRecord: (value: VmRecord, depth: number, options: SerializeOptions) => string;
    /** 序列化属性名 */
    serializePropName: (value: number | string, options: SerializeOptions) => string;
    /** 序列化函数 */
    serializeFunction: (value: VmFunction, options: SerializeOptions) => string;
    /** 序列化模块 */
    serializeModule: (value: VmModule, depth: number, options: SerializeOptions) => string;
    /** 序列化外部值 */
    serializeExtern: (value: VmExtern, depth: number, options: SerializeOptions) => string;
}
