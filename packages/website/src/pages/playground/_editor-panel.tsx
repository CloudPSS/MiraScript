import { useEffect, useRef, type JSX } from 'react';
import { configCheckpoint } from '@mirascript/mirascript';
import Editor from '@site/src/components/Mira/editor';
import { setMonacoContext } from '@site/src/components/Mira/monaco-context';
import { runMiraScript, type Results } from '@site/src/components/Mira/runner';
import { EXAMPLES } from './_examples';
import { globals } from './_globals';
import { usePlaygroundState } from './_state-manager';
import styles from './index.module.css';

/** 编辑面板属性 */
type EditorPanelProps = {
    setResults: React.Dispatch<React.SetStateAction<Results | null>>;
};

/** 编辑面板 */
export default function EditorPanel({ setResults }: EditorPanelProps): JSX.Element {
    const [state, setState] = usePlaygroundState();
    const lang = state.mode === 'Script' ? 'mirascript' : 'mirascript-template';
    const run = useRef<(source: string) => Promise<void>>(async () => {
        /* noop */
    });
    useEffect(() => {
        run.current = async (source: string) => {
            try {
                configCheckpoint(800);
                const results = await runMiraScript(source, state.mode, globals(), 'playground', true);
                setResults(results);
            } finally {
                configCheckpoint();
            }
        };
    }, [state.mode]);
    return (
        <>
            <div className={styles['editor-header']}>
                <h3>编辑器</h3>
                <label htmlFor="example-select">示例</label>
                <select
                    id="example-select"
                    className={styles['editor-options']}
                    onChange={(e) => {
                        const index = Number.parseInt(e.target.value, 10);
                        const example = EXAMPLES[index]!;
                        if (!example) return;
                        void example.code().then((code) => {
                            setState({ mode: example.mode, source: code });
                        });
                    }}
                >
                    <option value={-1}>-- 选择示例 --</option>
                    {EXAMPLES.map((example, index) => (
                        <option key={example.order} value={index}>
                            {example.name}
                        </option>
                    ))}
                </select>
                <label htmlFor="mode-select">模式</label>
                <select
                    id="mode-select"
                    className={styles['editor-options']}
                    onChange={(e) => {
                        const mode = e.target.value as 'Script' | 'Template';
                        setState({ mode });
                    }}
                >
                    <option value="Script" selected={state.mode === 'Script'}>
                        Script
                    </option>
                    <option value="Template" selected={state.mode === 'Template'}>
                        Template
                    </option>
                </select>
                <button className={styles['editor-options']} onClick={() => void run.current(state.source)} title="Ctrl+Enter">
                    运行
                </button>
            </div>
            <Editor
                wrapperProps={{ className: styles['editor-content'] }}
                language={lang}
                value={state.source}
                path="file:///playground"
                options={{
                    wordWrap: 'on',
                    wrappingIndent: 'indent',
                    minimap: { renderCharacters: false },
                }}
                onMount={(editor, monaco: typeof import('@private/monaco-editor')) => {
                    setMonacoContext(editor.getModel(), globals());
                    editor.addAction({
                        id: 'run-mirascript',
                        label: '运行',
                        keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
                        run: async (editor) => run.current(editor.getValue()),
                    });
                }}
                onChange={(value) => {
                    setState({ source: value });
                }}
            />
        </>
    );
}
