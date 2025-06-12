import type { IDisposable } from 'monaco-editor';
export type { IDisposable };
/** monaco editor */
export type Monaco = Readonly<typeof import('monaco-editor')>;

/** 加载器 */
export class MiraScriptMonacoLoader implements IDisposable {
    constructor(private readonly monaco: Monaco) {
        const { languages } = monaco;

        languages.register({
            id: 'mirascript',
            extensions: ['.mira'],
            aliases: ['MiraScript', 'mirascript', 'mira'],
            mimetypes: ['text/x-mirascript'],
        });

        languages.register({
            id: 'mirascript-template',
            extensions: ['.miratpl'],
            aliases: ['MiraScriptTemplate', 'mirascript-template', 'miratpl'],
            mimetypes: ['text/x-mirascript-template'],
        });

        const _loadBasicFeatures = () => void this.loadBasicFeatures();
        const _loadFullFeatures = () => void Promise.all([this.loadBasicFeatures(), this.loadLSPFeatures()]);

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
            this.disposables.push(...registerBasic(this.monaco));
        } catch (error) {
            // eslint-disable-next-line no-console
            console.error('Failed to load MiraScript basic features:', error);
        }
    }
    /** 加载 LSP 功能 */
    async loadLSPFeatures(): Promise<void> {
        try {
            const { registerLSP } = await import('./lsp/index.js');
            if (this._lspFeaturesLoaded || this.disposed) return;
            this._lspFeaturesLoaded = true;
            this.disposables.push(...registerLSP(this.monaco));
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
/** Api 泛化 */
// eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
type AsApi<T> = T extends Function
    ? // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
      Function
    : T extends object
      ? {
            [K in keyof T]: AsApi<T[K]>;
        }
      : T;
/** Monaco Api 属性 */
type MonacoApi = AsApi<Monaco>;
/**
 * 注册 MiraScript Monaco 编辑器扩展。
 */
export function registerMiraScript(monaco: MonacoApi): MiraScriptMonacoLoader {
    if (
        !monaco ||
        typeof monaco !== 'object' ||
        !monaco.languages ||
        'function' != typeof monaco.languages.register ||
        'function' != typeof monaco.languages.onLanguage ||
        !monaco.editor ||
        'function' != typeof monaco.editor.create ||
        'function' != typeof monaco.editor.createModel ||
        'function' != typeof monaco.editor.createWebWorker ||
        'function' != typeof monaco.Uri ||
        'function' != typeof monaco.Range ||
        'function' != typeof monaco.Position ||
        'function' != typeof monaco.CancellationTokenSource ||
        'function' != typeof monaco.Emitter
    ) {
        throw new TypeError('Invalid Monaco editor instance provided.');
    }
    return new MiraScriptMonacoLoader(monaco as Monaco);
}
