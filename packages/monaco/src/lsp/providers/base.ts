import type { VmContext, VmAny } from '@mirascript/mirascript';
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
    cachedContext = null;
}

/** 提供全局变量的执行上下文 */
export type MonacoContext = Readonly<Required<Pick<VmContext, 'get' | 'describe' | 'keys' | 'has'>>> & {
    /** 获取指定 key 的值，找不到时返回 undefined */
    getOrUndefined(key: string): VmAny;
};
const DEFAULT_MONACO_CONTEXT: MonacoContext = Object.freeze({
    get(key: string) {
        return DefaultVmContext.get(key);
    },
    getOrUndefined(key: string) {
        return DefaultVmContext.has(key) ? DefaultVmContext.get(key) : undefined;
    },
    has(key: string) {
        return DefaultVmContext.has(key);
    },
    keys() {
        return DefaultVmContext.keys();
    },
    describe(key: string) {
        return undefined;
    },
});
/** 获取执行上下文 */
async function getContextImpl(model: editor.ITextModel, clearCache: () => void): Promise<MonacoContext> {
    try {
        const context = await contextProvider?.(model);
        if (!context) {
            return DEFAULT_MONACO_CONTEXT;
        }

        const monacoContext: MonacoContext = Object.freeze({
            get(key: string) {
                try {
                    return context.get(key);
                } catch {
                    clearCache();
                    return DEFAULT_MONACO_CONTEXT.get(key);
                }
            },
            getOrUndefined(key: string) {
                try {
                    return context.has(key) ? context.get(key) : undefined;
                } catch {
                    clearCache();
                    return DEFAULT_MONACO_CONTEXT.getOrUndefined(key);
                }
            },
            has(key: string) {
                try {
                    return context.has(key);
                } catch {
                    clearCache();
                    return DEFAULT_MONACO_CONTEXT.has(key);
                }
            },
            keys() {
                try {
                    return context.keys();
                } catch {
                    clearCache();
                    return DEFAULT_MONACO_CONTEXT.keys();
                }
            },
            describe(key: string) {
                try {
                    return context.describe?.(key) || undefined;
                } catch {
                    clearCache();
                    return DEFAULT_MONACO_CONTEXT.describe(key);
                }
            },
        });
        return monacoContext;
    } catch (ex) {
        // eslint-disable-next-line no-console
        console.error('Error getting MiraScript Monaco context:', ex);
        return DEFAULT_MONACO_CONTEXT;
    }
}
let cachedContext: readonly [editor.ITextModel, Promise<MonacoContext>] | null = null;
/** 获取执行上下文 */
export async function getContext(model: editor.ITextModel): Promise<MonacoContext> {
    if (cachedContext?.[0] === model) {
        return cachedContext[1];
    }
    const clearCache = () => {
        if (cachedContext === cache) {
            cachedContext = null;
        }
    };
    const context = getContextImpl(model, clearCache);
    const cache = [model, context] as const;
    cachedContext = cache;
    setTimeout(clearCache, 100);
    return context;
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
    async getContext(model: editor.ITextModel): Promise<MonacoContext> {
        return getContext(model);
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
