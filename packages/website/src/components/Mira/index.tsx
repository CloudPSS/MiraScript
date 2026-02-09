import type { JSX } from 'react';
import type { InputMode } from '@mirascript/mirascript';
import styles from './styles.module.css';
import { Editor } from '../../client/monaco';

/** MiraScript 编辑器 */
export default function Mira({ value, mode, title }: { value: string; mode: InputMode; title: string }): JSX.Element {
    const lineCount = value.split('\n').length;
    const editor = (
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
                lineNumbersMinChars: Math.floor(Math.log10(lineCount)) + 3,
            }}
        />
    );
    return (
        <div className={styles['host']}>
            <div className={styles['editor-holder']}>{editor}</div>
            <pre className={styles['pre']}>
                <code>{value}</code>
            </pre>
        </div>
    );
}
