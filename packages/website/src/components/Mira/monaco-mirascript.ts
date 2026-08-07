import { getMonacoContext } from './monaco-context';

/** 注册 MiraScript */
export async function registerMiraScript(monaco: typeof import('@private/monaco-editor')): Promise<void> {
    const { registerMiraScript } = await import('@mirascript/monaco');
    const loader = registerMiraScript(monaco, getMonacoContext);
    loader.features.codeLens = false;
    await loader.loadBasicFeatures();
    monaco.editor.createModel('', 'mirascript').dispose();
    monaco.editor.createModel('', 'mirascript-template').dispose();
}
