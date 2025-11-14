import type { VmContext } from '@mirascript/mirascript';
import { DefaultVmContext, type IRange } from '@mirascript/mirascript/subtle';
import type { VmContextProvider } from '../../index.js';
import { type editor, Emitter, type IEvent, type IPosition } from '../../monaco-api.js';
import type { CompileResult, FieldsAccessAt, VariableAccessAt } from '../compile-result.js';
import { compile } from '../worker-helper.js';
import { wordAt } from '../utils.js';

let contextProvider: VmContextProvider | undefined;
/** 设置全局变量提供者 */
export function setContextProvider(provider: VmContextProvider | undefined): void {
    contextProvider = provider;
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
    /** 获取执行上下文（全局变量） */
    async getContext(model: editor.ITextModel): Promise<Readonly<VmContext>> {
        return (await contextProvider?.(model)) ?? DefaultVmContext;
    }
    /** 获取当前位置的值 */
    async getValueAt(
        model: editor.ITextModel,
        position: IPosition,
    ): Promise<({ range: IRange } & ({ variable: VariableAccessAt } | { fields: FieldsAccessAt })) | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const d = compiled.variableAccessAt(model, position);
        if (d) return { range: d.range, variable: d };
        const word = wordAt(model, position);
        if (word) {
            const a = compiled.fieldAccessAt(model, {
                lineNumber: position.lineNumber,
                column: word.range.endColumn,
            });
            if (a) return { range: word.range, fields: a };
        }
        return undefined;
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
