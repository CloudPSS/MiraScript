import type { TextDocument } from 'vscode';
import { load as parseYaml } from 'js-yaml';
import type { ModelAdapter } from '../adapter/model.js';
import type { VmFunctionOption, VmImmutable, VmPrimitive, VmValue } from '@mirascript/mirascript';
import { Disposable, workspace, mira, miraMonacoLsp, type Uri } from '#loader';
const { createVmContext, isVmArray, isVmExtern, isVmModule, VmExtern, VmFunction, VmModule } = mira;
const { Provider } = miraMonacoLsp;

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

/** 搜索配置文件 */
async function searchConfig(uri: Uri): Promise<Uri | null> {
    const configs = await workspace.findFiles(
        '**/.mirarc{,.json,.yaml,.yml,.js,.cjs,.mjs,.ts,.mts,.cts}',
        '**/node_modules/**',
    );
    if (!configs.length) {
        return null;
    }
    const candidates = [];
    for (const config of configs) {
        if (uri.scheme !== config.scheme || uri.authority !== config.authority) {
            continue;
        }
        if (uri.path.startsWith(config.path.replace(/\.mirarc(\..+)?$/, ''))) {
            candidates.push(config);
        }
    }
    if (candidates.length === 0) {
        return null;
    }
    candidates.sort((a, b) => b.path.length - a.path.length);
    return candidates[0]!;
}

/** 加载 Yaml/Json 配置 */
function loadYamlConfig(uri: Uri, data: string): unknown {
    const parsed = parseYaml(data);
    return parsed;
}
/** 加载 JS/TS 配置 */
async function loadJsConfig(uri: Uri, data: string): Promise<unknown> {
    const isTs = uri.path.endsWith('ts');
    const mime = isTs ? 'application/typescript' : 'application/javascript';
    const directUri = () => `${uri.toString()}?t=${Date.now()}`;
    const dataUri = () => {
        return `data:${mime},${encodeURIComponent(data)}?t=${Date.now()}`;
    };
    const blobUri = () => {
        const blob = new Blob([data], { type: mime });
        const url = URL.createObjectURL(blob);
        setTimeout(() => URL.revokeObjectURL(url), 1000);
        return url;
    };
    let module: unknown = null;
    try {
        module = await import(directUri());
    } catch {
        // ignore
    }
    try {
        module = await import(dataUri());
    } catch {
        // ignore
    }
    try {
        module = await import(blobUri());
    } catch {
        // ignore
    }
    if (module == null) {
        throw new Error(`Failed to load config from ${uri.toString()}`);
    }
    if (typeof module == 'object' && 'default' in module) {
        return module.default;
    }
    return module;
}
const decoder = new TextDecoder();
/** 加载配置 */
async function loadConfig(uri: Uri): Promise<MiraConfig> {
    const content = await workspace.fs.readFile(uri);
    const str = decoder.decode(content);
    if (
        uri.path.endsWith('.js') ||
        uri.path.endsWith('.cjs') ||
        uri.path.endsWith('.mjs') ||
        uri.path.endsWith('.ts') ||
        uri.path.endsWith('.mts') ||
        uri.path.endsWith('.cts')
    ) {
        return (await loadJsConfig(uri, str)) as MiraConfig;
    } else {
        return loadYamlConfig(uri, str) as MiraConfig;
    }
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
    private static createValue(value: VmImmutable | VmFunctionOption, maybeFunction?: boolean): VmImmutable {
        if (value == null || typeof value != 'object') {
            return value satisfies VmImmutable;
        }
        if (!maybeFunction || isVmArray(value) || isVmExtern(value) || isVmModule(value)) {
            return value satisfies VmImmutable;
        }

        return VmFunction(() => {
            throw new Error('Function is not implemented.');
        }, value as VmFunctionOption);
    }
    readonly globals = new Map<string, VmValue>();
    /** 加载配置 */
    async reload(): Promise<void> {
        let config: MiraConfig | undefined;
        if (this.document != null) {
            try {
                config = await loadConfig(this.document.uri);
            } catch {
                // ignore
            }
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
                VmFunction(
                    () => {
                        throw new Error('Function is not implemented.');
                    },
                    { ...value, name: key },
                ),
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

            const module: Record<string, VmImmutable> = Object.create(null);
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
        Provider.setContextProvider(async (model) => {
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
        const cached = this.cacheByFile.get(editor.uri.toString());
        if (cached) return cached;

        const config = await searchConfig(editor.uri);
        if (config == null) return MiraConfigData.default;

        let data = this.cacheByConfig.get(config.toString());
        if (data == null) {
            data = new MiraConfigData(await workspace.openTextDocument(config));
            await data.reload();
            this.cacheByConfig.set(config.toString(), data);
        }
        this.cacheByFile.set(editor.uri.toString(), data);
        return data;
    }
}
