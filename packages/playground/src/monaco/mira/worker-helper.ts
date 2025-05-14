import type * as wasm from 'mira-wasm';
import type { Res } from './worker.js';

let id = 0;
let worker: Promise<Worker> | undefined;
const calls = new Map<number, [(result: unknown) => void, (error: unknown) => void]>();
/** 准备 worker */
async function getWorker(): Promise<Worker> {
    if (worker) return worker;
    worker = new Promise((resolve, reject) => {
        const w = new Worker(new URL('./worker.js', import.meta.url), { name: '@mirascript/worker', type: 'module' });
        w.addEventListener('message', (ev) => {
            if (ev.data === 'ready') {
                resolve(w);
            } else {
                const [id, _, result] = ev.data as Res<keyof typeof wasm>;
                if (typeof id !== 'number') return;
                const call = calls.get(id);
                if (!call) return;
                calls.delete(id);
                const [resolve, reject] = call;
                if (result instanceof Error) {
                    reject(result);
                } else {
                    resolve(result);
                }
            }
        });
        w.addEventListener('error', (ev) => {
            reject(new Error(`Worker error: ${ev.message}`));
        });
    });
    return worker;
}

/** 调用 worker 方法 */
export async function callWorker<const M extends keyof typeof wasm>(
    method: M,
    ...args: Parameters<(typeof wasm)[M]>
): Promise<ReturnType<(typeof wasm)[M]>> {
    const w = await getWorker();
    const callId = id++;
    if (id >= Number.MAX_SAFE_INTEGER) id = 0;
    const transfer = [];
    for (const a of args) {
        if (ArrayBuffer.isView(a)) transfer.push(a.buffer);
    }
    w.postMessage([callId, method, ...args], { transfer });
    return new Promise((resolve, reject) => {
        calls.set(callId, [resolve as (result: unknown) => void, reject]);
    });
}
