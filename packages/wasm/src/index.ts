import * as wasm from '../lib/wasm.js';

export { DiagnosticCode, OpCode, CompileFlag } from '../lib/wasm.js';
export { wasm };

/** 编译结果 */
export interface CompileResult {
    /** 编译诊断 */
    readonly diagnostics: Uint32Array;
    /** 编译生成的字节码 */
    readonly chunk: Uint8Array | undefined;
}

/** 编译 */
function compileImpl(
    compiler: (script: Uint8Array, options: Uint8Array) => wasm.CompileResult,
    script: Uint8Array,
    flags: Uint8Array,
): CompileResult {
    const result = compiler(script, flags);
    try {
        const diagnostics = result.diagnostics();
        const chunk = result.chunk();
        return { diagnostics, chunk };
    } finally {
        result.free();
    }
}

/** 编译 MiraScript 脚本 */
export function compileScript(script: Uint8Array, flags: Uint8Array): CompileResult {
    return compileImpl(wasm.compile_script, script, flags);
}

/** 编译 MiraScript 插值字符串 */
export function compileTemplate(script: Uint8Array, flags: Uint8Array): CompileResult {
    return compileImpl(wasm.compile_template, script, flags);
}
