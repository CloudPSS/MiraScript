import * as wasm from 'mira-wasm';
import { initialize, type Uri, type worker } from '@private/monaco-editor/worker';

/** Host functions */
export interface Host {
    /** 更新编译结果 */
    updateCompileResult(uri: string, errors: ArrayBuffer, chunk: ArrayBuffer | undefined): void;
}

initialize((ctx: worker.IWorkerContext<Host>) => {
    context = ctx;
    return { ...wasm, compile_script };
});
let context: worker.IWorkerContext<Host>;

export { keywords, control_keywords, constant_keywords, numeric_keywords, get_error_message, opcodes } from 'mira-wasm';
/** 编译 */
export async function compile_script(uri: Uri): Promise<void> {
    const model = context.getMirrorModels().find((v) => v.uri.toString() === uri.toString());
    if (!model) {
        throw new Error(`Model not found for URI: ${uri.toString()}`);
    }
    const script = model.getValue();
    const result = wasm.compile_script(script);
    const error = result.errors().buffer;
    const chunk = result.chunk()?.buffer;
    result.free();
    await Promise.resolve(context.host.updateCompileResult(uri.toString(), error, chunk));
}
