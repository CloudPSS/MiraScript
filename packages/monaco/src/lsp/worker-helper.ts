import type { editor } from '../monaco-api.js';
import type { Ready, CacheKey, Req, Res, ResOk } from './worker.js';
import { CompileResult } from './compile-result.js';
import { setMarkers } from './diagnostics.js';

/** 缓存 */
type CacheValue = {
    readonly version: number;
    readonly result: Promise<CompileResult>;
    lastAccess: number;
};
/** 编译结果缓存，避免重复编译 */
const cache = new Map<CacheKey, CacheValue>();
let worker: Promise<Worker> | undefined = undefined;

// 清理缓存
const CACHE_MAX_AGE = 30000;
setInterval(() => {
    const now = Date.now();
    for (const [key, { lastAccess }] of cache) {
        if (now - lastAccess > CACHE_MAX_AGE) {
            cache.delete(key);
        }
    }
}, CACHE_MAX_AGE);

/** 使用 worker 进行编译 */
async function compileWorker(req: Req): Promise<CompileResult> {
    if (!worker) {
        const w = new Worker(new URL('#lsp/worker', import.meta.url), {
            type: 'module',
            name: '@mirascript/lsp-server',
        });
        worker = new Promise((resolve, reject) => {
            const onError = (e: ErrorEvent) => {
                cleanUp();
                reject(new Error(`Worker failed to start: ${e.message}`));
            };
            const onMessage = (e: MessageEvent<Ready | Error>) => {
                if (e.data === 'mirascript lsp ready') {
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
    }

    const instance = await worker;
    instance.postMessage(req);
    const [key, version, source] = req;
    const [_key, _version, result] = await new Promise<ResOk>((resolve, reject) => {
        const onMessage = (e: MessageEvent<Res>) => {
            if (e.data[0] === key && e.data[1] === version) {
                instance.removeEventListener('message', onMessage);
                if (e.data[2] instanceof Error) {
                    reject(e.data[2]);
                } else {
                    resolve(e.data as ResOk);
                }
            }
        };
        instance.addEventListener('message', onMessage);
    });
    return new CompileResult(key, version, source, result);
}

let compileImpl: typeof import('./worker.js').compile;
/** 使用当前线程编译 */
async function compileSync(req: Req): Promise<CompileResult> {
    const [key, version, script, mode] = req;
    if (compileImpl == null) {
        const mod = await import('./worker.js');
        compileImpl = mod.compile;
    }
    const result = compileImpl(script, mode);
    return new CompileResult(key, version, script, result);
}

const USE_WORKER = typeof Worker === 'function';
/** 注册 worker */
export async function compile(model: editor.ITextModel): Promise<CompileResult> {
    const uri = model.uri.toString();
    const version = model.getVersionId();
    const mode = model.getLanguageId() === 'mirascript-template' ? 'Template' : 'Script';
    const cacheKey = `${model.id}\0${uri}\0${mode}` as const;
    const cached = cache.get(cacheKey);
    if (cached?.version === version) {
        cached.lastAccess = Date.now();
        return cached.result;
    }

    const value = model.getValue();
    const req: Req = [cacheKey, version, value, mode];
    const res = USE_WORKER ? compileWorker(req) : compileSync(req);
    void res.then(async (result) => setMarkers(model, result));
    const item: CacheValue = {
        version,
        lastAccess: Date.now(),
        result: res,
    };
    cache.set(cacheKey, item);
    res.catch(() => {
        const current = cache.get(cacheKey);
        if (current === item) {
            cache.delete(cacheKey);
        }
    });
    return res;
}
