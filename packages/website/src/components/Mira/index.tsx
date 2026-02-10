import { useState, type JSX } from 'react';
import type { InputMode, VmAny } from '@mirascript/mirascript';
import { Editor, useMonaco } from './monaco';
import styles from './styles.module.css';
import { Highlight } from './highlight';
import type { Result } from './runner';

/** 结果显示 */
function ResultItem({ item }: { item: Result }): JSX.Element {
    const H = (item: Result['content'][number], i: number): JSX.Element => {
        if (typeof item === 'string') {
            return (
                <span key={i} className={styles['result-content']}>
                    {item}
                </span>
            );
        }
        return <Highlight key={i} code={item.value} language="mirascript" />;
    };
    if (item.type === 'error') {
        return <code className={`${styles['result-item']} ${styles['result-error']}`}>{item.content.map((c, i) => H(c, i))}</code>;
    } else if (item.type === 'log') {
        return <code className={`${styles['result-item']} ${styles['result-log']}`}>{item.content.map((c, i) => H(c, i))}</code>;
    } else {
        return <code className={`${styles['result-item']} ${styles['result-result']}`}>{item.content.map((c, i) => H(c, i))}</code>;
    }
}

/** 结果显示 */
function Results({ results, outdated }: { results: Result[]; outdated: boolean }): JSX.Element {
    return (
        <pre className={`${styles['results']} ${outdated ? styles['results-outdated'] : ''}`}>
            {results.map((item, index) => (
                <ResultItem key={index} item={item} />
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
    const monaco = useMonaco();
    const editor = monaco && (
        <div className={styles['editor-holder']}>
            <Editor
                path={title ? `title:///#${encodeURIComponent(title)}` : undefined}
                className={styles['editor']}
                value={value}
                theme="vs-dark"
                language={mode === 'Template' ? 'mirascript-template' : mode === 'Doc' ? 'mirascript-doc' : 'mirascript'}
                options={{
                    scrollBeyondLastLine: false,
                    minimap: { enabled: false },
                    formatOnType: true,
                    formatOnPaste: true,
                    fontSize: 14.4,
                    lineHeight: 14.4 * 1.45,
                    fontFamily: 'var(--ifm-font-family-monospace)',
                    fontLigatures: true,
                    automaticLayout: true,
                    lineDecorationsWidth: 0,
                    lineNumbersMinChars: Math.floor(Math.log10(lineCount)) + 3,
                    tabSize: 2,
                    readOnly: readOnly,
                    'semanticHighlighting.enabled': true,
                }}
                onMount={(editor) => {
                    if (mode !== 'Doc') {
                        editor.addAction({
                            id: 'run-mirascript',
                            label: '运行',
                            keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
                            run: (editor) => {
                                void import('./runner').then(async ({ runMiraScript }) => {
                                    const code = editor.getValue();
                                    const results = await runMiraScript(code, mode, context);
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
