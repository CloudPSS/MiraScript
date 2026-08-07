import type { CompiledArtifact } from '@site/src/components/Mira/runner';
import type { PythonSourceRequest, PythonSourceResponse } from './_python-source-protocol';

let worker: Worker | undefined;
let requestId = 0;
const pending = new Map<
    number,
    {
        /** 完成请求。 */
        resolve(source: string): void;
        /** 拒绝请求。 */
        reject(error: Error): void;
    }
>();

/** 获取 Python 代码生成 worker。 */
function getWorker(): Worker {
    if (worker) return worker;
    worker = new Worker(new URL('./_python-source-worker.ts', import.meta.url), { type: 'module' });
    worker.addEventListener('message', (event: MessageEvent<PythonSourceResponse>) => {
        const response = event.data;
        const request = pending.get(response.id);
        if (!request) return;
        pending.delete(response.id);
        if (response.error == null) request.resolve(response.source);
        else request.reject(new Error(response.error));
    });
    worker.addEventListener('error', (event) => {
        const error = new Error(event.message || 'Python source worker failed.');
        for (const request of pending.values()) request.reject(error);
        pending.clear();
        worker?.terminate();
        worker = undefined;
    });
    return worker;
}

/** 使用 Pyodide 生成 artifact 对应的 Python 源代码。 */
export async function generatePythonSource(artifact: CompiledArtifact, assetsUrl: string): Promise<string> {
    const id = ++requestId;
    const request: PythonSourceRequest = {
        id,
        assetsUrl,
        source: artifact.source,
        mode: artifact.mode,
        fileName: artifact.fileName,
    };
    return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject });
        getWorker().postMessage(request);
    });
}
