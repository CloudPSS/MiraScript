import { useState, type JSX } from 'react';
import Layout from '@theme/Layout';
import Editor from '@site/src/components/Mira/editor';
import ResultItem from '@site/src/components/Mira/result';
import { setMonacoContext } from '@site/src/components/Mira/monaco-context';
import { runMiraScript, type Result } from '@site/src/components/Mira/runner';
import { usePlaygroundState } from './_state-manager';
import { EXAMPLES } from './_examples';
import { globals } from './_globals';
import { configCheckpoint } from '@mirascript/mirascript';
import styles from './index.module.css';

/** 编辑面板 */
function EditorPanel({ setResults }: { setResults: React.Dispatch<React.SetStateAction<Result[]>> }): JSX.Element {
    const [state, setState] = usePlaygroundState();
    const lang = state.mode === 'Script' ? 'mirascript' : 'mirascript-template';
    const run = async (source: string) => {
        try {
            configCheckpoint(800);
            const results = await runMiraScript(source, state.mode, globals(), 'playground', true);
            setResults(results);
        } finally {
            configCheckpoint();
        }
    };
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
                <button className={styles['editor-options']} onClick={() => void run(state.source)} title="Ctrl+Enter">
                    运行
                </button>
            </div>
            <Editor
                className={styles['editor-content']}
                language={lang}
                value={state.source}
                keepCurrentModel
                path={`playground.${lang === 'mirascript' ? 'mira' : 'miratpl'}`}
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
                        run: async (editor) => run(editor.getValue()),
                    });
                }}
                onChange={(value) => {
                    setState({ source: value });
                }}
            />
        </>
    );
}

/** 输出面板 */
function OutputPanel({ results }: { results: Result[] }): JSX.Element {
    return (
        <>
            <div className={styles['output-header']}>
                <h3>输出</h3>
            </div>
            <div className={styles['output-content']}>
                {results.map((result, index) => (
                    <ResultItem key={index} item={result} styles={styles} showTimestamp />
                ))}
            </div>
        </>
    );
}

/**
 * 在线编辑器页面
 */
export default function Playground(): JSX.Element {
    const [results, setResults] = useState<Result[]>([]);
    return (
        <Layout wrapperClassName={styles['root']} title="在线编辑器" description="通过浏览器在线编写并运行 MiraScript 代码">
            <EditorPanel setResults={setResults} />
            <OutputPanel results={results} />
        </Layout>
    );
}
