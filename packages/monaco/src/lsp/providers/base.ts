import type { VmGlobal } from 'mirascript';
import { VmSharedGlobal } from 'mirascript/subtle';
import type { VmGlobalProvider } from '../../index.js';
import { type editor, Emitter, type IEvent } from '../../monaco-api.js';
import type { CompileResult } from '../compile-result.js';
import { compile } from '../worker-helper.js';

let globalProvider: VmGlobalProvider | undefined;
/** 设置全局变量提供者 */
export function setGlobalProvider(provider: VmGlobalProvider | undefined): void {
    globalProvider = provider;
}

/** 提供编辑器 LSP 支持 */
export abstract class Provider {
    /** 获取编译结果 */
    async getCompileResult(model: editor.ITextModel): Promise<CompileResult | undefined> {
        if (model.uri.scheme === 'mirascript') {
            return undefined; // 不处理标准库
        }
        return await compile(model);
    }
    /** 获取全局变量 */
    async getGlobals(model: editor.ITextModel): Promise<Readonly<VmGlobal>> {
        return (await globalProvider?.(model)) ?? VmSharedGlobal;
    }

    readonly displayName = 'MiraScript LSP';
    readonly _debugDisplayName = 'MiraScript LSP';
    private _onDidChange: Emitter<this> | null = null;
    /** @inheritdoc */
    get onDidChange(): IEvent<this> & IEvent<void> {
        this._onDidChange ??= new Emitter<this>();
        return this._onDidChange.event as IEvent<this> & IEvent<void>;
    }
    /** 触发 onDidChange */
    emitDidChange(): void {
        this._onDidChange?.fire(this);
    }
}
