import { useId, useState, type JSX } from 'react';
import type { InputMode, VmAny } from '@mirascript/mirascript';
import type { Result } from './runner';
import { getMonacoContext, setMonacoContext } from './monaco-context';
import { useMonaco } from './monaco';
import Editor from './editor';
import styles from './index.module.css';
import ResultItem from './result';

/** 结果显示 */
function Results({ results, outdated }: { results: Result[]; outdated: boolean }): JSX.Element {
    return (
        <pre className={`${styles['results']} ${outdated ? styles['results-outdated'] : ''}`}>
            {results.map((item, index) => (
                <ResultItem key={index} item={item} styles={styles} />
            ))}
        </pre>
    );
}

/** MiraScript 编辑器 */
export default function Mira({
    value,
    mode,
    title,
    readOnly,
    autoRun,
    context,
}: {
    value: string;
    mode: InputMode | 'Doc';
    title?: string;
    readOnly?: boolean;
    autoRun?: boolean;
    context?: Record<string, VmAny>;
}): JSX.Element {
    const lineCount = value.split('\n').length;
    const [results, setResults] = useState<Result[]>([]);
    const [resultsOutdated, setResultsOutdated] = useState(true);
    const language = mode === 'Template' ? 'mirascript-template' : mode === 'Doc' ? 'mirascript-doc' : 'mirascript';
    const path = `markdown:///${useId()}/live-editor${title ? `?title=${encodeURIComponent(title)}` : ''}`;
    const editor = useMonaco() && (
        <div className={styles['editor-holder']}>
            <Editor
                path={path}
                className={styles['editor']}
                value={value}
                theme="vs-dark"
                language={language}
                loading={<></>}
                options={{
                    scrollBeyondLastLine: false,
                    minimap: { enabled: false },
                    scrollbar: { ignoreHorizontalScrollbarInContentHeight: true, alwaysConsumeMouseWheel: false },
                    fontSize: 14.4,
                    lineHeight: 14.4 * 1.45,
                    lineDecorationsWidth: 0,
                    lineNumbersMinChars: Math.floor(Math.log10(lineCount)) + 3,
                    readOnly: readOnly,
                }}
                onMount={(editor, monaco: typeof import('@private/monaco-editor')) => {
                    setMonacoContext(editor.getModel(), context);

                    if (mode !== 'Doc') {
                        editor.addAction({
                            id: 'run-mirascript',
                            label: '运行',
                            keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
                            run: (editor) => {
                                void import('./runner').then(async ({ runMiraScript }) => {
                                    const code = editor.getValue();
                                    const results = await runMiraScript(code, mode, getMonacoContext(editor.getModel()!));
                                    setResults(results);
                                    setResultsOutdated(code !== editor.getValue());
                                });
                            },
                        });
                        if (autoRun) {
                            void editor.getAction('run-mirascript')?.run();
                        }
                    }
                }}
                onChange={() => setResultsOutdated(true)}
            />
        </div>
    );
    const hasResults = results.length > 0;
    return (
        <>
            <div className={`${styles['host']} ${hasResults ? styles['with-results'] : ''}`}>
                {editor}
                <pre className={styles['pre']}>
                    <code>{value} </code>
                </pre>
            </div>
            {hasResults ? <Results results={results} outdated={resultsOutdated} /> : null}
        </>
    );
}
