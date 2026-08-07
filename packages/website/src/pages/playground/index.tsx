import { useState, type JSX } from 'react';
import Layout from '@theme/Layout';
import type { Results } from '@site/src/components/Mira/runner';
import EditorPanel from './_editor-panel';
import ResultPanel from './_result-panel';
import styles from './index.module.css';

/**
 * 在线编辑器页面
 */
export default function Playground(): JSX.Element {
    const [result, setResult] = useState<Results | null>(null);
    return (
        <Layout wrapperClassName={styles['root']} title="在线编辑器" description="通过浏览器在线编写并运行 MiraScript 代码">
            <EditorPanel setResults={setResult} />
            <ResultPanel results={result} />
        </Layout>
    );
}
