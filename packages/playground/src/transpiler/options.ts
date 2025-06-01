/** 代码编译选项 */
export interface CompileOptions {
    /** 错误报告级别 */
    readonly errorLevel?: 'none' | 'info' | 'warning' | 'error';
}
/** 代码生成选项 */
export interface GenerateOptions {
    /** 是否美化代码 */
    readonly pretty?: boolean;
}

/** 转换选项 */
export type TranspileOptions = GenerateOptions & CompileOptions;
