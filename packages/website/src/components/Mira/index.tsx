import type { JSX } from 'react';
import { useInView } from 'react-intersection-observer';
import type { InputMode } from '@mirascript/mirascript';
import styles from './styles.module.css';
import { Editor } from '../../client/monaco';

/** MiraScript 编辑器 */
export default function Mira({ value, mode, title }: { value: string; mode: InputMode; title: string }): JSX.Element {
    const { ref, inView } = useInView({ triggerOnce: true });
    const editor = inView ? (
        <Editor
            path={title ? `title:///#${encodeURIComponent(title)}` : undefined}
            className={styles['editor']}
            value={value}
            theme="vs-dark"
            language={mode === 'Template' ? 'mirascript-template' : 'mirascript'}
            options={{
                scrollBeyondLastLine: false,
                minimap: { enabled: false },
                fontSize: 14.4,
                lineHeight: 14.4 * 1.45,
                fontFamily: 'var(--ifm-font-family-monospace)',
                readOnly: true,
                lineDecorationsWidth: 0,
                lineNumbersMinChars: 3,
            }}
        />
    ) : null;
    return (
        <div className={styles['host']}>
            <div className={styles['editor-holder']} ref={ref}>
                {editor}
            </div>
            <pre className={styles['pre']}>
                <code>{value}</code>
            </pre>
        </div>
    );
}
