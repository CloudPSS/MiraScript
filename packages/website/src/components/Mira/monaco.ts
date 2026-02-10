import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';
import type { languages, Uri } from '@private/monaco-editor';
import { useEffect, useState } from 'react';
import { loader } from '@monaco-editor/react';

/** 加载 monaco */
async function loadMonaco(): Promise<typeof import('@private/monaco-editor')> {
    const monaco = await import('@private/monaco-editor');
    monaco.editor.setTheme('vs-dark');

    const { registerMiraScript } = await import('@mirascript/monaco');
    const loader = registerMiraScript(monaco);
    loader.features.codeLens = false;
    await loader.loadBasicFeatures();
    monaco.editor.createModel('', 'mirascript').dispose();
    monaco.editor.createModel('', 'mirascript-template').dispose();

    monaco.editor.registerCommand('run-mirascript', (_, uri: Uri) => {
        const model = monaco.editor.getModel(uri);
        if (!model) return;
        const editor = monaco.editor.getEditors().find((e) => e.getModel() === model);
        if (!editor) return;
        void editor.getAction('run-mirascript')?.run();
    });
    monaco.languages.registerCodeLensProvider(['mirascript', 'mirascript-template', 'mirascript-doc'], {
        provideCodeLenses(model, token) {
            const { uri } = model;
            const title = uri.scheme === 'title' ? uri.fragment : '';
            const lenses: languages.CodeLens[] = [];
            if (title) {
                lenses.push({
                    range: { startLineNumber: 1, startColumn: 1, endLineNumber: 1, endColumn: 1 },
                    id: 'title',
                    command: {
                        id: '',
                        title: title,
                        arguments: [model.uri],
                    },
                });
            }
            if (model.getLanguageId() !== 'mirascript-doc') {
                lenses.push({
                    range: { startLineNumber: 1, startColumn: 1, endLineNumber: 1, endColumn: 1 },
                    id: 'run-mirascript',
                    command: {
                        id: 'run-mirascript',
                        title: '运行 (Ctrl+Enter)',
                        arguments: [model.uri],
                    },
                });
            } else if (!lenses.length) {
                lenses.push({
                    range: { startLineNumber: 1, startColumn: 1, endLineNumber: 1, endColumn: 1 },
                    id: 'placeholder',
                    command: {
                        id: '',
                        title: '',
                        arguments: [model.uri],
                    },
                });
            }
            return { lenses };
        },
    });
    return monaco;
}

const monacoModule = ExecutionEnvironment.canUseDOM ? loadMonaco() : null;
loader.config({ monaco: monacoModule as never });

/** 使用 Monaco 编辑器 */
export function useMonaco(): typeof import('@private/monaco-editor') | null {
    const [monaco, setMonaco] = useState<typeof import('@private/monaco-editor') | null>(null);
    useEffect(() => {
        void monacoModule?.then(setMonaco);
    }, []);

    return monaco;
}

export { Editor, DiffEditor } from '@monaco-editor/react';
