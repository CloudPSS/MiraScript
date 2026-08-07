import { useState, type JSX } from 'react';
import ResultItem from '@site/src/components/Mira/result';
import SourceViewer from './_source-viewer';
import type { Results } from '@site/src/components/Mira/runner';
import PythonSourceViewer from './_python-source';
import styles from './index.module.css';

/** 输出面板页签 */
type OutputTab = 'output' | 'javascript' | 'python';
const TABS: Record<OutputTab, string> = {
    javascript: 'JavaScript 源代码',
    python: 'Python 源代码',
    output: '控制台',
};

/** 输出面板属性 */
type ResultPanelProps = {
    results: Results | null;
};

/** 输出面板内容 */
function ResultPanelContent({ results, tab }: ResultPanelProps & { tab: OutputTab }): JSX.Element {
    if (tab === 'output') {
        if (!results) return <div className={styles['compiled-placeholder']}>运行代码后将在这里显示输出结果。</div>;
        return (
            <>
                {results.items.map((result, index) => (
                    <ResultItem key={index} item={result} styles={styles} showTimestamp />
                ))}
            </>
        );
    }
    if (!results) {
        const lang = (
            {
                javascript: 'JavaScript',
                python: 'Python',
            } as const
        )[tab];
        return <div className={styles['compiled-placeholder']}>运行代码后将在这里显示编译生成的 {lang}。</div>;
    }
    if (!results.javascript) {
        return <div className={styles['compiled-placeholder']}>编译失败，请查看控制台输出。</div>;
    }
    switch (tab) {
        case 'javascript':
            return <SourceViewer language="javascript" source={results.javascript} path="file:///playground.js" />;
        case 'python':
            return <PythonSourceViewer artifact={results} />;
    }
}

/** 输出面板 */
export default function ResultPanel({ results }: ResultPanelProps): JSX.Element {
    const [tab, setTab] = useState<OutputTab>('output');
    const tabs = Object.entries(TABS).map(([value, label]) => (
        <button
            key={value}
            className={`${styles['output-tab']} ${tab === value ? styles['output-tab-active'] : ''}`}
            role="tab"
            aria-selected={tab === value}
            onClick={() => setTab(value as OutputTab)}
        >
            {label}
        </button>
    ));

    return (
        <>
            <div className={styles['output-header']}>
                <h3>输出</h3>
                <div className={styles['output-tabs']} role="tablist" aria-label="输出内容">
                    {tabs}
                </div>
            </div>
            <div className={styles['output-content']}>
                <ResultPanelContent results={results} tab={tab} />
            </div>
        </>
    );
}
