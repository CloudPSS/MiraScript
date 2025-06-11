import type { VmScript } from '../vm/index.js';
import type { TranspileOptions, ScriptInput } from './types.js';
import { compile as compileCore } from './compile.js';
import { createScript } from './create-script.js';
import { compileFast } from './compile-fast.js';

export type { TranspileOptions, ScriptInput, ParseMode } from './types.js';

// 目前编译速度约 200kB/s
const WORKER_MIN_LEN = 1024;

let worker: Promise<Worker> | undefined;
/** 获取 worker */
async function getWorker(): Promise<Worker> {
    if (worker) return worker;
    worker = new Promise((resolve, reject) => {
        const w = new Worker(new URL('#compiler/worker', import.meta.url), {
            type: 'module',
            name: '@mirascript/compiler',
        });
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
async function compileWorker(
    ...args: Parameters<typeof compileCore>
): Promise<[Uint8Array, string | undefined, Uint32Array]> {
    const worker = await getWorker();
    const seq = Math.random();
    worker.postMessage([seq, ...args]);
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
async function compileImpl(...args: Parameters<typeof compileCore>): Promise<VmScript> {
    const [_, code, errors] =
        args[0].length < WORKER_MIN_LEN ? await compileCore(...args) : await compileWorker(...args);
    if (!code) {
        throw new Error(`Failed to compile: ${errors.join(', ')}`);
    }
    return createScript(args[0], code);
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(source: ScriptInput, options: TranspileOptions = {}): Promise<VmScript> {
    if (typeof source == 'string') {
        const result = compileFast(source, options);
        if (result) {
            return result;
        }
    }
    return compileImpl(source, options);
}
