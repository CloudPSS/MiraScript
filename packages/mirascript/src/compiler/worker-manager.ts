import type { compile } from './worker.js';

let worker: Promise<Worker> | undefined;
/** 获取 worker */
async function getWorker(): Promise<Worker> {
    if (worker) return worker;
    worker = new Promise((resolve, reject) => {
        const w = new Worker(new URL('#compiler/worker', import.meta.url), {
            type: 'module',
            name: '@mirascript/compiler',
        });
        const onError = (e: ErrorEvent) => {
            cleanUp();
            reject(new Error(`Worker failed to start: ${e.message}`));
        };
        const onMessage = (e: MessageEvent<'ready' | Error>) => {
            if (e.data === 'ready') {
                cleanUp();
                resolve(w);
            } else if (e.data instanceof Error) {
                cleanUp();
                reject(e.data);
            }
        };
        w.addEventListener('error', onError);
        w.addEventListener('message', onMessage);
        const cleanUp = () => {
            w.removeEventListener('error', onError);
            w.removeEventListener('message', onMessage);
        };
        setTimeout(() => {
            onError(new ErrorEvent('error', { message: 'Worker did not respond in time' }));
        }, 30000);
    });
    return worker;
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compileWorker(...args: Parameters<typeof compile>): Promise<[string | undefined, Uint32Array]> {
    const worker = await getWorker();
    const seq = Math.random();
    worker.postMessage([seq, ...args]);
    return await new Promise<[string | undefined, Uint32Array]>((resolve, reject) => {
        const callback = (ev: MessageEvent) => {
            const data = ev.data as [number, string | undefined, Uint32Array] | [number, string];
            if (!Array.isArray(data)) return;
            const [retSeq, ...rest] = data;
            if (seq !== retSeq) return; // Ignore messages not matching the request
            worker.removeEventListener('message', callback);
            if (rest.length === 2) {
                resolve(rest);
            } else {
                reject(new Error(rest[0]));
            }
        };
        worker.addEventListener('message', callback);
    });
}
