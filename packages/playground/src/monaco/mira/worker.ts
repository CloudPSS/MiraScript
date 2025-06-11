import * as wasm from '@mirascript/wasm';
import { initialize, type Uri, type worker } from '@private/monaco-editor/worker';
import { toCompileFlags } from 'mirascript/subtle';

/** Host functions */
export interface Host {
    /** 更新编译结果 */
    updateCompileResult(uri: string, version: number, diagnostics: ArrayBuffer, chunk: ArrayBuffer | undefined): void;
}

export const exports = {
    compileScript: async (uri: Uri): Promise<void> => compileImpl(uri, wasm.compileScript),
    compileTemplate: async (uri: Uri): Promise<void> => compileImpl(uri, wasm.compileTemplate),
};

initialize((ctx: worker.IWorkerContext<Host>) => {
    context = ctx;
    return exports;
});

let context: worker.IWorkerContext<Host>;
let compileCache: [uri: string, version: number, promise: Promise<void>] = ['', Number.NaN, Promise.resolve()];

const flags = toCompileFlags({
    UseUtf16: true,
    TrackReferences: true,
    HideDiagnosticOther: false,
});
const encoder = new TextEncoder();
/** 编译 */
async function compileImpl(uri: Uri, compiler = wasm.compileScript): Promise<void> {
    const reqUri = uri.toString();
    const model = context.getMirrorModels().find((v) => v.uri.toString() === reqUri);
    if (!model) {
        throw new Error(`Model not found for URI: ${uri.toString()}`);
    }
    const { version } = model;
    if (compileCache[0] === reqUri && compileCache[1] === version) {
        return compileCache[2];
    }
    const script = model.getValue();
    const result = compiler(encoder.encode(script), flags);
    compileCache = [
        reqUri,
        version,
        Promise.resolve(context.host.updateCompileResult(uri.toString(), version, result.diagnostics, result.chunk)),
    ];
}
