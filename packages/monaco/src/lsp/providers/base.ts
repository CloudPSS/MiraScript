import type { editor, Emitter, IEvent } from '@private/monaco-editor';
import type { ParseMode } from 'mirascript';
import type { Monaco } from '../..';
import { CompileResult } from '../compile-result';
import { setMarkers } from '../diagnostics';
import { callWorker } from '../worker-helper';

/** 提供编辑器 LSP 支持 */
export abstract class Provider {
    private static readonly pendingRequests: Array<{
        monaco: Monaco;
        uri: string;
        mode: ParseMode;
        version: number;
        resolve: (result: CompileResult | undefined) => void;
    }> = [];
    /** 写入编译结果 */
    static updateCompileResult(
        monaco: Monaco,
        uri: string,
        mode: ParseMode,
        version: number,
        diagnosticsBuffer: ArrayBuffer,
        chunkBuffer?: ArrayBuffer,
    ): void {
        const result = CompileResult.set(monaco, uri, mode, version, diagnosticsBuffer, chunkBuffer);
        const model = monaco.editor.getModel(monaco.Uri.parse(uri));
        if (model) setMarkers(model, result);
        // 处理挂起的请求
        for (let i = Provider.pendingRequests.length - 1; i >= 0; i--) {
            const request = Provider.pendingRequests[i]!;
            if (request.monaco === monaco && request.uri === uri && request.version === version) {
                Provider.pendingRequests.splice(i, 1);
                request.resolve(result);
            }
        }
    }
    /** 获取编译结果 */
    async getCompileResult(model: editor.ITextModel, currentVersion = true): Promise<CompileResult | undefined> {
        if (model.uri.scheme === 'mirascript') {
            return undefined; // 不处理标准库
        }
        const { monaco } = this;
        if (model.getEOL() !== '\n') {
            // 确保使用 LF 作为行结束符
            model.setEOL(monaco.editor.EndOfLineSequence.LF);
        }
        return new Promise((resolve) => {
            const uri = model.uri.toString();
            const mode = model.getLanguageId() === 'mirascript-template' ? 'template' : 'script';
            const version = currentVersion ? model.getVersionId() : undefined;
            const result = CompileResult.get(monaco, uri, mode, version);
            if (result) {
                return resolve(result);
            }
            void callWorker(this.monaco, mode === 'template' ? 'compileTemplate' : 'compileScript', model.uri).catch(
                (ex) => {
                    // eslint-disable-next-line no-console
                    console.error(ex);
                },
            );
            setTimeout(() => {
                resolve(undefined);
            }, 10000);
            Provider.pendingRequests.push({
                monaco,
                uri,
                mode,
                version: version ?? model.getVersionId(),
                resolve,
            });
        });
    }

    constructor(protected readonly monaco: Monaco) {}
    readonly displayName = 'MiraScript LSP';
    private _onDidChange: Emitter<this> | null = null;
    /** @inheritdoc */
    get onDidChange(): IEvent<this> & IEvent<void> {
        this._onDidChange ??= new this.monaco.Emitter<this>();
        return this._onDidChange.event as IEvent<this> & IEvent<void>;
    }
    /** 触发 onDidChange */
    emitDidChange(): void {
        this._onDidChange?.fire(this);
    }
}
