import type { JSX } from 'react';
import type { Result } from './runner';
import Highlight from './highlight';
import styles from './result.module.css';

/** 提取 HTML 结果 */
function htmlResult(item: Result): string | undefined {
    if (item.type !== 'result' || item.content.length !== 1) return undefined;
    const content = item.content[0];
    if (typeof content === 'object' && typeof content.raw === 'string' && content.raw.toLowerCase().startsWith('<!doctype html>')) {
        return content.raw;
    }
    return undefined;
}

/** 展示一条结果 */
export default function ResultItem({
    item,
    styles: cStyles,
    showTimestamp,
}: {
    item: Result;
    styles: Record<string, string>;
    showTimestamp?: boolean;
}): JSX.Element {
    const html = htmlResult(item);
    let inner;
    let empty = false;
    if (html) {
        // 特殊处理 HTML 输出
        inner = <iframe srcDoc={html}></iframe>;
    } else {
        const H = (item: Result['content'][number], i: number): JSX.Element | null => {
            if (typeof item === 'string') {
                if (!item) return null;
                return <span key={i}>{item}</span>;
            } else {
                if (!item.value) return null;
                return <Highlight key={i} code={item.value} language="mirascript" />;
            }
        };
        inner = item.content.map((c, i) => H(c, i));
        empty = inner.every((e) => e === null);
    }
    return (
        <div className={`${styles['root']} ${showTimestamp ? styles['with-timestamp'] : ''} ${cStyles['result-item']} ${cStyles[`result-${item.type}`]}`}>
            {showTimestamp && <time>{item.timestamp.toFixed(2)}</time>}
            {inner}
            {empty && <span className={styles['empty']}> </span>}
        </div>
    );
}
