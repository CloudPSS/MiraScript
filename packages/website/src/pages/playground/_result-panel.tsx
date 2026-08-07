import { useEffect, useRef, useState, type JSX } from 'react';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Editor from '@site/src/components/Mira/editor';
import ResultItem from '@site/src/components/Mira/result';
import type { CompiledArtifact, Result } from '@site/src/components/Mira/runner';
import { generatePythonSource } from './_python-source';
import styles from './index.module.css';

/** 输出面板页签 */
type OutputTab = 'output' | 'javascript' | 'python';

/** Python 源代码页签状态。 */
type PythonState =
    | { status: 'idle'; artifact: CompiledArtifact | null }
    | { status: 'loading'; artifact: CompiledArtifact }
    | { status: 'ready'; artifact: CompiledArtifact; source: string }
    | { status: 'error'; artifact: CompiledArtifact; message: string };

/** 输出面板属性 */
type ResultPanelProps = {
    results: Result[];
    compiledArtifact: CompiledArtifact | null;
};

/** 只读源码编辑器。 */
function SourceEditor({ language, source, path }: { language: string; source: string; path: string }): JSX.Element {
    return (
        <Editor
            wrapperProps={{ className: styles['compiled-editor'] }}
            language={language}
            value={source}
            path={path}
            options={{
                readOnly: true,
                minimap: { enabled: false },
                wordWrap: 'on',
                wrappingIndent: 'deepIndent',
            }}
        />
    );
}

/** 输出面板 */
export default function ResultPanel({ results, compiledArtifact }: ResultPanelProps): JSX.Element {
    const [tab, setTab] = useState<OutputTab>('output');
    const [pythonState, setPythonState] = useState<PythonState>({ status: 'idle', artifact: null });
    const currentArtifact = useRef(compiledArtifact);
    const generationId = useRef(0);
    const assetsUrl = useBaseUrl('/pyodide.g.assets/');

    useEffect(() => {
        currentArtifact.current = compiledArtifact;
        generationId.current++;
        setPythonState({ status: 'idle', artifact: compiledArtifact });
    }, [compiledArtifact]);

    useEffect(() => {
        if (tab !== 'python' || !compiledArtifact || pythonState.status !== 'idle') return;
        const id = ++generationId.current;
        setPythonState({ status: 'loading', artifact: compiledArtifact });
        void generatePythonSource(compiledArtifact, assetsUrl).then(
            (source) => {
                if (currentArtifact.current === compiledArtifact && generationId.current === id) {
                    setPythonState({ status: 'ready', artifact: compiledArtifact, source });
                }
            },
            (error: unknown) => {
                if (currentArtifact.current === compiledArtifact && generationId.current === id) {
                    setPythonState({
                        status: 'error',
                        artifact: compiledArtifact,
                        message: error instanceof Error ? error.message : String(error),
                    });
                }
            },
        );
    }, [assetsUrl, compiledArtifact, pythonState.status, tab]);

    let content: JSX.Element | JSX.Element[];
    if (tab === 'output') {
        content = results.map((result, index) => <ResultItem key={index} item={result} styles={styles} showTimestamp />);
    } else if (!compiledArtifact) {
        content = <div className={styles['compiled-placeholder']}>运行代码后将在这里显示编译生成的{tab === 'javascript' ? ' JavaScript' : ' Python'}。</div>;
    } else if (tab === 'javascript') {
        content = <SourceEditor language="javascript" source={compiledArtifact.javascript} path="file:///playground.js" />;
    } else if (pythonState.status === 'ready' && pythonState.artifact === compiledArtifact) {
        content = <SourceEditor language="python" source={pythonState.source} path="file:///playground.py" />;
    } else if (pythonState.status === 'error' && pythonState.artifact === compiledArtifact) {
        content = (
            <div className={styles['compiled-placeholder']}>
                <div className={styles['compiled-error']}>{pythonState.message}</div>
                <button onClick={() => setPythonState({ status: 'idle', artifact: compiledArtifact })}>重试</button>
            </div>
        );
    } else {
        content = <div className={styles['compiled-placeholder']}>正在加载 Pyodide 并生成 Python 源代码…</div>;
    }

    return (
        <>
            <div className={styles['output-header']}>
                <h3>输出</h3>
                <div className={styles['output-tabs']} role="tablist" aria-label="输出内容">
                    {(
                        [
                            ['output', '输出'],
                            ['javascript', 'JS 源代码'],
                            ['python', 'Python 源代码'],
                        ] as const
                    ).map(([value, label]) => (
                        <button
                            key={value}
                            className={`${styles['output-tab']} ${tab === value ? styles['output-tab-active'] : ''}`}
                            role="tab"
                            aria-selected={tab === value}
                            onClick={() => setTab(value)}
                        >
                            {label}
                        </button>
                    ))}
                </div>
            </div>
            <div className={styles['output-content']}>{content}</div>
        </>
    );
}
