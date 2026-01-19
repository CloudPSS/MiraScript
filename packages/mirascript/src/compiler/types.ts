import type { Config } from '@mirascript/constants';
export type { InputMode, ScriptInput } from '@mirascript/constants';

/** 代码编译选项 */
export type CompileOptions = Config;
/** 代码生成选项 */
export interface GenerateOptions {
    /** 是否美化代码 */
    readonly pretty?: boolean;
    /** 是否生成源映射 */
    readonly sourceMap?: boolean;
    /** 代码文件名 */
    readonly fileName?: string;
}
/** 转换选项 */
export type TranspileOptions = GenerateOptions & CompileOptions;
