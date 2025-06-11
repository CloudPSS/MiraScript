import type { exports, Host } from './worker.js';
import { utils, editor, Uri, Emitter, type IEvent } from '@private/monaco-editor';
import { CompileResult } from './compile-result.js';

/** 提供编辑器 LSP 支持 */
export class Provider {
    private static readonly pendingRequests: Array<{
        uri: string;
        version: number;
        resolve: (result: CompileResult | undefined) => void;
    }> = [];
    /** 写入编译结果 */
    static updateCompileResult(
        uri: string,
        version: number,
        diagnosticsBuffer: ArrayBuffer,
        chunkBuffer?: ArrayBuffer,
    ): void {
        const result = CompileResult.set(uri, version, diagnosticsBuffer, chunkBuffer);
        // 处理挂起的请求
        for (let i = Provider.pendingRequests.length - 1; i >= 0; i--) {
            const request = Provider.pendingRequests[i]!;
            if (request.uri === uri && request.version === version) {
                Provider.pendingRequests.splice(i, 1);
                request.resolve(result);
            }
        }
    }
    /** 获取编译结果 */
    static async getCompileResult(model: editor.ITextModel, currentVersion = true): Promise<CompileResult | undefined> {
        if (model.uri.scheme === 'mirascript') {
            return undefined; // 不处理标准库
        }
        if (model.getEOL() !== '\n') {
            // 确保使用 LF 作为行结束符
            model.setEOL(editor.EndOfLineSequence.LF);
        }
        return new Promise((resolve) => {
            const uri = model.uri.toString();
            const version = currentVersion ? model.getVersionId() : undefined;
            const result = CompileResult.get(uri, version);
            if (result) {
                return resolve(result);
            }
            void callWorker('compileScript', model.uri).catch((ex) => {
                // eslint-disable-next-line no-console
                console.error(ex);
            });
            setTimeout(() => {
                resolve(undefined);
            }, 10000);
            Provider.pendingRequests.push({
                uri,
                version: version ?? model.getVersionId(),
                resolve,
            });
        });
    }
    readonly displayName = 'MiraScript LSP';
    private readonly _onDidChange = new Emitter<this>();
    /** @inheritdoc */
    get onDidChange(): IEvent<this> & IEvent<void> {
        return this._onDidChange.event as IEvent<this> & IEvent<void>;
    }
    /** 触发 onDidChange */
    emitDidChange(): void {
        this._onDidChange.fire(this);
    }
}

utils.registerWorker(
    'mirascript',
    () => new Worker(new URL('./worker.js', import.meta.url), { name: '@mirascript/lsp-server', type: 'module' }),
);
const monacoWorker = editor.createWebWorker({
    label: 'mirascript',
    moduleId: '@mirascript/lsp-server',
    createData: {},
    host: {
        updateCompileResult: (uri, version, diagnosticsBuffer, chunkBuffer) => {
            Provider.updateCompileResult(uri, version, diagnosticsBuffer, chunkBuffer);
        },
    } satisfies Host,
});

/** 调用 worker 方法 */
export async function callWorker<const M extends keyof typeof exports>(
    method: M,
    ...args: Parameters<(typeof exports)[M]>
): Promise<ReturnType<(typeof exports)[M]>> {
    const resources = args.filter((a: unknown) => a instanceof Uri);
    const passArgs = args.map((arg: unknown) => (arg instanceof Uri ? arg.toString() : arg));
    const proxy = resources.length ? await monacoWorker.withSyncedResources(resources) : await monacoWorker.getProxy();
    return await ((proxy as typeof exports)[method as 'compileScript'](...(passArgs as [Uri])) as unknown as Promise<
        ReturnType<(typeof exports)[M]>
    >);
}
