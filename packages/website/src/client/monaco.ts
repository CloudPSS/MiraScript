import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';
import { useEffect, useMemo, useState } from 'react';
import { useColorMode } from '@docusaurus/theme-common';
import { loader } from '@monaco-editor/react';

const monacoModule = ExecutionEnvironment.canUseDOM
    ? Promise.resolve().then(async () => {
          const monaco = await import('@private/monaco-editor');
          const { registerMiraScript } = await import('@mirascript/monaco');
          const loader = registerMiraScript(monaco);
          await loader.loadBasicFeatures();
          monaco.editor.createModel('', 'mirascript').dispose();
          monaco.editor.createModel('', 'mirascript-template').dispose();
          return monaco;
      })
    : null;
loader.config({ monaco: monacoModule as never });

/** 使用 Monaco 编辑器 */
export function useMonaco(): typeof import('@private/monaco-editor') | null {
    const [monaco, setMonaco] = useState<typeof import('@private/monaco-editor') | null>(null);
    const { colorMode } = useColorMode();
    useEffect(() => {
        void monacoModule?.then(setMonaco);
    }, []);
    useMemo(() => {
        monaco?.editor.setTheme(colorMode === 'dark' ? 'vs-dark' : 'vs');
    }, [monaco, colorMode]);

    return monaco;
}

export { Editor, DiffEditor } from '@monaco-editor/react';
