import type { CompileFlag } from '@mirascript/wasm';

/** 代码编译选项 */
export type CompileOptions = Readonly<Partial<Record<Exclude<keyof typeof CompileFlag, 'MAX'>, boolean>>>;
/** 代码生成选项 */
export interface GenerateOptions {
    /** 是否美化代码 */
    readonly pretty?: boolean;
}

/** 转换选项 */
export type TranspileOptions = GenerateOptions & CompileOptions;

/** 解析模式 */
export type ParseMode = 'script' | 'template';

/**
 * 编译输入，支持字符串和 UTF-8 字节数组
 */
export type ScriptInput = string | Uint8Array;
