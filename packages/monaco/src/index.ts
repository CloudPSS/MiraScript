import type { VmGlobal } from 'mirascript';
import { type editor, languages, registerMonacoApi, type IDisposable, type MonacoApi } from './monaco-api.js';
import { register } from './contribute.js';

/** 提供全局变量信息 */
export type VmGlobalProvider = (model: editor.ITextModel) => languages.ProviderResult<Readonly<VmGlobal>>;

/** 加载器 */
export class MiraScriptMonacoLoader implements IDisposable {
    constructor(readonly globalProvider?: VmGlobalProvider) {
        register();

        const _loadBasicFeatures = () => void this.loadBasicFeatures();
        const _loadFullFeatures = () => void this.loadLSPFeatures();

        languages.onLanguageEncountered('mirascript', _loadBasicFeatures);
        languages.onLanguage('mirascript', _loadFullFeatures);

        languages.onLanguageEncountered('mirascript-template', _loadBasicFeatures);
        languages.onLanguage('mirascript-template', _loadFullFeatures);
    }
    private _basicFeaturesLoaded = false;
    private _lspFeaturesLoaded = false;
    /** 加载基础功能 */
    async loadBasicFeatures(): Promise<void> {
        try {
            const { registerBasic } = await import('./basic/index.js');
            if (this._basicFeaturesLoaded || this.disposed) return;
            this._basicFeaturesLoaded = true;
            this.disposables.push(...registerBasic());
        } catch (error) {
            // eslint-disable-next-line no-console
            console.error('Failed to load MiraScript basic features:', error);
        }
    }
    /** 加载 LSP 功能 */
    async loadLSPFeatures(): Promise<void> {
        try {
            const basic = this.loadBasicFeatures();
            const { registerLSP } = await import('./lsp/index.js');
            await basic; // 确保基础功能已加载
            if (this._lspFeaturesLoaded || this.disposed) return;
            this._lspFeaturesLoaded = true;
            this.disposables.push(...registerLSP(this.globalProvider));
        } catch (error) {
            // eslint-disable-next-line no-console
            console.error('Failed to load MiraScript LSP features:', error);
        }
    }

    private readonly disposables: IDisposable[] = [];
    private disposed = false;
    /** @inheritdoc */
    dispose(): void {
        if (this.disposed) {
            throw new Error('MiraScriptMonacoLoader has already been disposed.');
        }
        this.disposed = true;
        for (const d of this.disposables.splice(0)) {
            d.dispose();
        }
    }
}
export { registerMonacoApi, register };
/**
 * 注册 MiraScript Monaco 编辑器扩展。
 */
export function registerMiraScript(monacoApi: MonacoApi, globalProvider?: VmGlobalProvider): MiraScriptMonacoLoader {
    registerMonacoApi(monacoApi);
    return new MiraScriptMonacoLoader(globalProvider);
}
