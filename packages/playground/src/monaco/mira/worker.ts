import * as wasm from 'mira-wasm';
import { initialize, type Uri, type worker } from '@private/monaco-editor/worker';

/** Host functions */
export interface Host {
    /** 更新编译结果 */
    updateCompileResult(uri: string, version: number, errors: ArrayBuffer, chunk: ArrayBuffer | undefined): void;
}

initialize((ctx: worker.IWorkerContext<Host>) => {
    context = ctx;
    return { ...wasm, compile_script };
});
let context: worker.IWorkerContext<Host>;

export { keywords, control_keywords, constant_keywords, numeric_keywords, get_error_message } from 'mira-wasm';

let compileCache: [uri: string, version: number, promise: Promise<void>] = ['', Number.NaN, Promise.resolve()];
/** 编译 */
export async function compile_script(uri: Uri): Promise<void> {
    const reqUri = uri.toString();
    const model = context.getMirrorModels().find((v) => v.uri.toString() === reqUri);
    if (!model) {
        throw new Error(`Model not found for URI: ${uri.toString()}`);
    }
    const { version } = model;
    if (compileCache[0] === reqUri && compileCache[1] === version) {
        return await compileCache[2];
    }
    const script = model.getValue();
    const compiling = compileImpl(reqUri, version, script);
    compileCache = [reqUri, version, compiling];
    await compiling;
}

/** 编译 */
async function compileImpl(uri: string, version: number, script: string): Promise<void> {
    const result = wasm.compile_script(script);
    const diagnostics = result.diagnostics().buffer;
    const chunk = result.chunk()?.buffer;
    result.free();
    await Promise.resolve(context.host.updateCompileResult(uri.toString(), version, diagnostics, chunk));
}
