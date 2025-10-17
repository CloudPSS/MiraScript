import { Disposable, workspace, type TextDocument } from 'vscode';
import { cosmiconfig } from 'cosmiconfig';
import { setContextProvider } from '@mirascript/monaco/lsp';
import type { ModelAdapter } from '../adapter/model.js';
import {
    createVmContext,
    isVmArray,
    isVmExtern,
    isVmModule,
    VmExtern,
    VmFunction,
    VmModule,
    type VmFunctionOption,
    type VmPrimitive,
    type VmValue,
} from '@mirascript/mirascript';

const explorer = cosmiconfig('mira', {
    searchStrategy: 'project',
    cache: false,
});

/** 配置信息 */
export interface MiraConfig {
    /** 全局变量 */
    globals: Record<string, VmValue>;
    /** 全局函数定义 */
    functions: Record<string, VmFunctionOption | null>;
    /** 全局模块 */
    modules: Record<string, Record<string, VmFunctionOption | VmPrimitive>>;
    /** 全局外部对象 */
    externs: Record<string, object>;
}

/** 配置信息 */
class MiraConfigData extends Disposable {
    static default: MiraConfigData = new MiraConfigData(undefined);

    constructor(readonly document: TextDocument | undefined) {
        const disposables: Disposable[] = [];
        if (document != null) {
            disposables.push(
                workspace.onDidSaveTextDocument(async (event) => {
                    if (event === document) {
                        await this.reload();
                    }
                }),
            );
        }
        super(() => {
            for (const disposable of disposables) {
                disposable.dispose();
            }
            this.globals.clear();
        });
    }
    /** 创建值 */
    private static createValue(value: VmValue | VmFunctionOption, maybeFunction?: boolean): VmValue {
        if (value == null || typeof value != 'object') {
            return value;
        }
        if (!maybeFunction || isVmArray(value) || isVmExtern(value) || isVmModule(value)) {
            return value as VmValue;
        }

        return VmFunction(() => {
            throw new Error('Function is not implemented.');
        }, value as VmFunctionOption);
    }
    readonly globals = new Map<string, VmValue>();
    /** 加载配置 */
    async reload(config?: MiraConfig): Promise<void> {
        if (config == null && this.document != null) {
            config = (await explorer.load(this.document.uri.fsPath))?.config as MiraConfig;
        }

        if (config == null) return;
        this.globals.clear();

        const globals = config.globals ?? {};
        for (const key in globals) {
            const value = globals[key];
            if (value === undefined) continue;

            this.globals.set(key, value);
        }
        const functions = config.functions ?? {};
        for (const key in functions) {
            const value = functions[key];
            if (value === undefined) continue;

            this.globals.set(
                key,
                VmFunction(() => {
                    throw new Error('Function is not implemented.');
                }, value ?? {}),
            );
        }
        const externs = config.externs ?? {};
        for (const key in externs) {
            const value = externs[key];
            if (value === undefined) continue;

            const vm =
                value != null && typeof value === 'object' ? new VmExtern(value) : MiraConfigData.createValue(value);
            this.globals.set(key, vm);
        }
        const modules = config.modules ?? {};
        for (const moduleName in modules) {
            const value = modules[moduleName];
            if (value == null || typeof value != 'object') continue;

            const module: Record<string, VmValue> = Object.create(null);
            for (const key in value) {
                const func = value[key];
                if (func === undefined) continue;
                module[key] = MiraConfigData.createValue(func, true);
            }
            this.globals.set(moduleName, new VmModule(moduleName, module));
        }
    }
}

/** 配置信息 */
export class ConfigManager extends Disposable {
    protected readonly disposables: Disposable[] = [];

    constructor() {
        super(() => {
            for (const disposable of this.disposables) {
                disposable.dispose();
            }
            for (const data of this.cacheByConfig.values()) {
                data.dispose();
            }
            this.cacheByConfig.clear();
            this.cacheByFile.clear();
        });
        setContextProvider(async (model) => {
            const { document } = model as ModelAdapter;
            const config = await this.config(document);
            return createVmContext(
                (key) => config.globals.get(key),
                () => config.globals.keys(),
            );
        });
    }

    readonly cacheByFile = new Map<string, MiraConfigData>();
    readonly cacheByConfig = new Map<string, MiraConfigData>();
    /** 查找配置 */
    async config(editor: TextDocument): Promise<MiraConfigData> {
        const filePath = editor.uri.fsPath;
        const cached = this.cacheByFile.get(filePath);
        if (cached) return cached;

        const config = await explorer.search(filePath);
        if (config == null) return MiraConfigData.default;

        let data = this.cacheByConfig.get(config.filepath);
        if (data == null) {
            data = new MiraConfigData(await workspace.openTextDocument(config.filepath));
            await data.reload(config.config as MiraConfig);
            this.cacheByConfig.set(config.filepath, data);
        }
        this.cacheByFile.set(filePath, data);
        return data;
    }
}
