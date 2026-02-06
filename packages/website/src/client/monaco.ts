import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';
import { useEffect, useState } from 'react';
import { loader } from '@monaco-editor/react';

const monacoModule = ExecutionEnvironment.canUseDOM
    ? Promise.resolve().then(async () => {
          const monaco = await import('@private/monaco-editor');
          const { registerMiraScript } = await import('@mirascript/monaco');
          const loader = registerMiraScript(monaco);
          loader.features.codeLens = false;
          await loader.loadBasicFeatures();
          monaco.editor.createModel('', 'mirascript').dispose();
          monaco.editor.createModel('', 'mirascript-template').dispose();
          monaco.languages.registerCodeLensProvider(['mirascript', 'mirascript-template'], {
              provideCodeLenses(model, token) {
                  const { uri } = model;
                  const title = uri.scheme === 'title' ? uri.fragment : '';
                  return {
                      lenses: [
                          {
                              range: { startLineNumber: 1, startColumn: 1, endLineNumber: 1, endColumn: 1 },
                              id: 'title',
                              command: { id: '', title: title, arguments: [model.uri] },
                          },
                      ],
                  };
              },
          });
          return monaco;
      })
    : null;
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
