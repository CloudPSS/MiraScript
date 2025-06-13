import { editor, Emitter, type IEvent } from '../../monaco-api.js';
import type { CompileResult } from '../compile-result';
import { compile } from '../worker-helper';

/** 提供编辑器 LSP 支持 */
export abstract class Provider {
    /** 获取编译结果 */
    async getCompileResult(model: editor.ITextModel): Promise<CompileResult | undefined> {
        if (model.uri.scheme === 'mirascript') {
            return undefined; // 不处理标准库
        }
        if (model.getEOL() !== '\n') {
            // 确保使用 LF 作为行结束符
            model.setEOL(editor.EndOfLineSequence.LF);
        }
        return await compile(model);
    }

    readonly displayName = 'MiraScript LSP';
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
