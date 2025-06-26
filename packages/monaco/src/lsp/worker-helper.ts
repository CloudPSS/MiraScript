import type { editor } from '../monaco-api.js';
import type { InputMode } from 'mirascript';
import type { Ready, Req, Res, ResOk } from './worker.js';
import { CompileResult } from './compile-result.js';
import { setMarkers } from './diagnostics.js';

const cache = new Map<`${string}\0${number}\0${InputMode}`, Promise<CompileResult>>();
let worker: Promise<Worker> | undefined = undefined;

/** 使用 worker 进行编译 */
async function compileWorker(req: Req): Promise<CompileResult> {
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

    const instance = await worker;
    instance.postMessage(req);
    const [uri, version] = req;
    const [_uri, _version, result] = await new Promise<ResOk>((resolve, reject) => {
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
    return new CompileResult(uri, version, result);
}

/** 使用当前线程编译 */
async function compileSync(req: Req): Promise<CompileResult> {
    const [uri, version, script, mode] = req;
    const { compile } = await import('./worker.js');
    const result = await compile(script, mode);
    return new CompileResult(uri, version, result);
}

const USE_WORKER = typeof Worker === 'function';
/** 注册 worker */
export async function compile(model: editor.ITextModel): Promise<CompileResult> {
    const uri = model.uri.toString();
    const version = model.getVersionId();
    const mode = model.getLanguageId() === 'mirascript-template' ? 'Template' : 'Script';
    const cacheKey = `${uri}\0${version}\0${mode}` as const;
    const cached = cache.get(cacheKey);
    if (cached) {
        return cached;
    }

    const value = model.getValue();
    const req: Req = [uri, version, value, mode];
    const res = USE_WORKER ? compileWorker(req) : compileSync(req);
    void res.then((result) => {
        setMarkers(model, result);
    });
    cache.set(cacheKey, res);
    res.catch(() => {
        cache.delete(cacheKey);
    }).finally(() => {
        // 清理缓存，避免内存泄漏
        setTimeout(() => {
            cache.delete(cacheKey);
        }, 10000);
    });
    return res;
}
