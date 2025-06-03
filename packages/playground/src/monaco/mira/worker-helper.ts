import type { DiagnosticCode } from 'mira-wasm';
import type * as worker from './worker.js';
import { utils, editor, Uri, Emitter, type IEvent } from '@private/monaco-editor';

/** 源代码诊断信息 */
export interface SourceDiagnostic {
    /** 代码 */
    readonly code: DiagnosticCode;
    /** 位置 */
    readonly startLineNumber: number;
    /** 位置 */
    readonly startColumn: number;
    /** 位置 */
    readonly endLineNumber: number;
    /** 位置 */
    readonly endColumn: number;
}

/** 编译结果 */
export interface CompileResult {
    /** 代码版本 */
    readonly version: number;
    /** 源代码诊断信息 */
    readonly diagnostics: readonly SourceDiagnostic[];
    /** 代码信息 */
    readonly chunk?: Uint8Array;
}

const compileResult = new Map<string, CompileResult>();

/** 获取编译结果 */
function getCompileResult(uri: string, version?: number): CompileResult | undefined {
    const data = compileResult.get(uri);
    if (data && version != null) {
        return data.version === version ? data : undefined;
    }
    return data;
}
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
        errorsBuffer: ArrayBuffer,
        chunkBuffer?: ArrayBuffer,
    ): void {
        const errors = new Uint32Array(errorsBuffer);
        const chunk = chunkBuffer ? new Uint8Array(chunkBuffer) : undefined;
        const diagnostics: SourceDiagnostic[] = [];
        for (let i = 0; i < errors.length; i += 5) {
            const startLineNumber = errors[i]!;
            const startColumn = errors[i + 1]!;
            const endLineNumber = errors[i + 2]!;
            const endColumn = errors[i + 3]!;
            const error = errors[i + 4]! as DiagnosticCode;
            diagnostics.push({
                code: error,
                startLineNumber,
                startColumn,
                endLineNumber,
                endColumn,
            });
        }
        const result: CompileResult = {
            version,
            diagnostics,
            chunk,
        };
        compileResult.set(uri, result);
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
    static async getCompileResult(model: editor.ITextModel): Promise<CompileResult | undefined> {
        return new Promise((resolve) => {
            const uri = model.uri.toString();
            const version = model.getVersionId();
            const result = getCompileResult(uri, version);
            if (result) {
                return resolve(result);
            }
            void callWorker('compile_script', model.uri).catch((ex) => {
                // eslint-disable-next-line no-console
                console.error(ex);
            });
            setTimeout(() => {
                resolve(undefined);
            }, 10000);
            Provider.pendingRequests.push({
                uri,
                version,
                resolve,
            });
        });
    }
    readonly displayName = 'MiraScript LSP';
    private readonly _onDidChange = new Emitter<void>();
    /** @inheritdoc */
    get onDidChange(): IEvent<void> | undefined {
        return this._onDidChange.event;
    }
    /** 触发 onDidChange */
    emitDidChange(): void {
        this._onDidChange.fire();
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
        updateCompileResult: (uri, version, errorsBuffer, chunkBuffer) => {
            Provider.updateCompileResult(uri, version, errorsBuffer, chunkBuffer);
        },
    } satisfies worker.Host,
});
/** 调用 worker 方法 */
export async function callWorker<const M extends keyof typeof worker>(
    method: M,
    ...args: Parameters<(typeof worker)[M]>
): Promise<ReturnType<(typeof worker)[M]>> {
    const resources = args.filter((a: unknown) => a instanceof Uri);
    const passArgs = args.map((arg: unknown) => (arg instanceof Uri ? arg.toString() : arg));
    const proxy = resources.length ? await monacoWorker.withSyncedResources(resources) : await monacoWorker.getProxy();
    return await ((proxy as typeof worker)[method as 'keywords'](...(passArgs as [])) as unknown as Promise<
        ReturnType<(typeof worker)[M]>
    >);
}
