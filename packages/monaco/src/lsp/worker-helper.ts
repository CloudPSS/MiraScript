import type { editor } from 'monaco-editor';
import type { ParseMode } from 'mirascript';
import type { Monaco } from '../index.js';
import type { Ready, Req, Res, ResOk } from './worker.js';
import { CompileResult } from './compile-result.js';
import { setMarkers } from './diagnostics.js';

const cache = new Map<`${string}\0${number}\0${ParseMode}`, Promise<CompileResult>>();
let worker: Promise<Worker> | undefined = undefined;
/** 注册 worker */
export async function compile(monaco: Monaco, model: editor.ITextModel): Promise<CompileResult> {
    if (!worker) {
        const w = new Worker(new URL('#/lsp/worker', import.meta.url), {
            name: '@mirascript/lsp-server',
            type: 'module',
        });
        worker = new Promise((resolve, reject) => {
            const onError = (e: ErrorEvent) => {
                w.removeEventListener('error', onError);
                w.removeEventListener('message', onMessage);
                reject(new Error(`Worker failed to start: ${e.message}`));
            };
            const onMessage = (e: MessageEvent<Ready>) => {
                if (e.data !== 'mirascript lsp ready') {
                    return;
                }
                w.removeEventListener('error', onError);
                w.removeEventListener('message', onMessage);
                resolve(w);
            };
            w.addEventListener('error', onError);
            w.addEventListener('message', onMessage);
            setTimeout(() => {
                onError(new ErrorEvent('error', { message: 'Worker did not respond in time' }));
            }, 30000);
        });
    }
    const uri = model.uri.toString();
    const version = model.getVersionId();
    const mode = model.getLanguageId() === 'mirascript-template' ? 'template' : 'script';
    const cacheKey = `${uri}\0${version}\0${mode}` as const;
    const cached = cache.get(cacheKey);
    if (cached) {
        return cached;
    }

    const value = model.getValue();
    const req = worker.then(async (instance) => {
        instance.postMessage([uri, version, value, mode] satisfies Req);
        const [_uri, _version, chunk, diagnostics] = await new Promise<ResOk>((resolve, reject) => {
            const onMessage = (e: MessageEvent<Res>) => {
                if (e.data[0] === uri && e.data[1] === version) {
                    instance.removeEventListener('message', onMessage);
                    if (typeof e.data[2] === 'string') {
                        reject(new Error(e.data[2]));
                    } else {
                        resolve(e.data as ResOk);
                    }
                }
            };
            instance.addEventListener('message', onMessage);
        });
        const result = new CompileResult(uri, version, diagnostics, chunk);
        setMarkers(monaco, model, result);
        return result;
    });
    cache.set(cacheKey, req);
    req.catch(() => {
        cache.delete(cacheKey);
    }).finally(() => {
        // 清理缓存，避免内存泄漏
        setTimeout(() => {
            cache.delete(cacheKey);
        }, 10000);
    });
    return req;
}
