import { useState, type JSX } from 'react';
import Editor from '@site/src/components/Mira/editor';
import ResultItem from '@site/src/components/Mira/result';
import type { Result } from '@site/src/components/Mira/runner';
import styles from './index.module.css';

/** 输出面板页签 */
type OutputTab = 'output' | 'source';

/** 输出面板属性 */
type ResultPanelProps = {
    results: Result[];
    compiledSource: string | null;
};

/** 输出面板 */
export default function ResultPanel({ results, compiledSource }: ResultPanelProps): JSX.Element {
    const [tab, setTab] = useState<OutputTab>('output');
    return (
        <>
            <div className={styles['output-header']}>
                <h3>输出</h3>
                <div className={styles['output-tabs']} role="tablist" aria-label="输出内容">
                    <button
                        className={`${styles['output-tab']} ${tab === 'output' ? styles['output-tab-active'] : ''}`}
                        role="tab"
                        aria-selected={tab === 'output'}
                        onClick={() => setTab('output')}
                    >
                        输出
                    </button>
                    <button
                        className={`${styles['output-tab']} ${tab === 'source' ? styles['output-tab-active'] : ''}`}
                        role="tab"
                        aria-selected={tab === 'source'}
                        onClick={() => setTab('source')}
                    >
                        JS 源代码
                    </button>
                </div>
            </div>
            <div className={styles['output-content']}>
                {tab === 'output' ? (
                    results.map((result, index) => <ResultItem key={index} item={result} styles={styles} showTimestamp />)
                ) : compiledSource == null ? (
                    <div className={styles['compiled-placeholder']}>运行代码后将在这里显示编译生成的 JavaScript。</div>
                ) : (
                    <Editor
                        wrapperProps={{ className: styles['compiled-editor'] }}
                        language="javascript"
                        value={compiledSource}
                        path="file:///playground.js"
                        options={{
                            readOnly: true,
                            minimap: { enabled: false },
                            wordWrap: 'on',
                            wrappingIndent: 'deepIndent',
                        }}
                    />
                )}
            </div>
        </>
    );
}
