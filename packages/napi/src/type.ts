/** NAPI 模块 */
export interface NapiModule {
    /** 编译 Mira 脚本 */
    compile(this: void, script: string | Uint8Array, config: object): Promise<CompileResult>;
    /** 编译 Mira 脚本 */
    compileSync(this: void, script: string | Uint8Array, config: object): CompileResult;
}

/** 编译结果 */
export interface CompileResult {
    /** 编译后的代码块 */
    chunk: Uint8Array;
    /** 编译期间的诊断信息 */
    diagnostics: Uint32Array;
}
