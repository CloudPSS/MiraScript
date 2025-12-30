import { monaco } from './loader.js';
import { currentTheme, onThemeChange } from './theme.js';

/** 创建编辑器实例 */
export function createEditor(
    container: HTMLElement,
    options: import('@private/monaco-editor').editor.IStandaloneEditorConstructionOptions = {},
): import('@private/monaco-editor').editor.IStandaloneCodeEditor {
    const overlay = monaco.utils.createOverflowWidgetsDomNode(container);
    const editor = monaco.editor.create(container, {
        fontFamily: 'var(--code-font)',
        fontLigatures: true,
        automaticLayout: true,
        useShadowDOM: true,
        overflowWidgetsDomNode: overlay,
        theme: currentTheme() === 'dark' ? 'vs-dark' : 'vs', // 初始主题，会在后面更新
        tabSize: 2,
        'semanticHighlighting.enabled': true,
        ...options,
    });
    editor.onDidDispose(() => overlay.dispose());
    onThemeChange((theme) => {
        monaco.editor.setTheme(theme === 'dark' ? 'vs-dark' : 'vs');
    });
    return editor;
}
