import type { CompileFlag } from 'mira-wasm';

/** 代码编译选项 */
export type CompileOptions = Readonly<Partial<Record<Exclude<keyof typeof CompileFlag, 'MAX'>, boolean>>>;
/** 代码生成选项 */
export interface GenerateOptions {
    /** 是否美化代码 */
    readonly pretty?: boolean;
}

/** 转换编译选项为标志位 */
export function toCompileFlags(options: CompileOptions, compileFlag: typeof CompileFlag): Uint8Array {
    const flags = new Uint8Array(Math.ceil(compileFlag.MAX / 8));
    for (const [key, value] of Object.entries(options)) {
        if (!value) continue;
        const index = compileFlag[key as keyof typeof CompileFlag];
        if (index == null) continue;
        flags[index / 8]! |= 1 << index % 8;
    }
    return flags;
}

/** 转换选项 */
export type TranspileOptions = GenerateOptions & CompileOptions;
