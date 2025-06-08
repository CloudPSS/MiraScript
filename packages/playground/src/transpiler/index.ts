import type { TranspileOptions } from './options.js';
import type { VmScript } from '../vm/index.js';
import { transpileCore } from './transpile.js';
import { createScript } from './create-script.js';

// 目前编译速度约 200kB/s
const WORKER_MIN_LEN = 1024;

let worker: Promise<Worker> | undefined;
/** 获取 worker */
async function getWorker(): Promise<Worker> {
    if (worker) return worker;
    worker = new Promise((resolve, reject) => {
        const w = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module', name: '@mirascript/compiler' });
        w.addEventListener('error', reject);
        w.addEventListener('message', (msg) => {
            if (msg.data === 'ready') {
                resolve(w);
            }
        });
    });
    return worker;
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
async function transpileWorker(
    code: string,
    options: TranspileOptions,
): Promise<[Uint8Array, string | undefined, Uint32Array]> {
    const worker = await getWorker();
    const seq = Math.random();
    worker.postMessage([seq, code, options]);
    return await new Promise<[Uint8Array, string | undefined, Uint32Array]>((resolve, reject) => {
        const callback = (ev: MessageEvent) => {
            const data = ev.data as [number, Uint8Array, string | undefined, Uint32Array] | [number, undefined, string];
            if (!Array.isArray(data)) return;
            const [retSeq, ...rest] = data;
            if (seq !== retSeq) return; // Ignore messages not matching the request
            worker.removeEventListener('message', callback);
            if (rest[0] == null) {
                reject(new Error(rest[1]));
            } else {
                resolve(rest);
            }
        };
        worker.addEventListener('message', callback);
    });
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function transpile(source: string, options: TranspileOptions = {}): Promise<VmScript> {
    const [_, code, errors] =
        source.length < WORKER_MIN_LEN ? await transpileCore(source, options) : await transpileWorker(source, options);
    if (!code) {
        throw new Error(`Failed to transpile code: ${errors.join(', ')}`);
    }
    return createScript(source, code);
}
