import { useEffect, useState, type JSX } from 'react';
import { useInView } from 'react-intersection-observer';
import type { InputMode } from '@mirascript/mirascript';
import { useColorMode } from '@docusaurus/theme-common';
import styles from './styles.module.css';
import { useMonaco } from '../../client/monaco';
import { Editor } from '../../client/monaco';

/** 加载高亮 */
function useHighlighting(value: string, language: string) {
    const monaco = useMonaco();
    const { colorMode } = useColorMode();
    const [highlighted, setHighlighted] = useState<string>(value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;'));

    useEffect(() => {
        if (!monaco) return;
        void (async () => {
            const html = await monaco.editor.colorize(value, language, {});
            setHighlighted(html);
        })();
    }, [monaco, language, value, colorMode]);

    return highlighted;
}

/** MiraScript 编辑器 */
export default function Mira({ value, mode }: { value: string; mode: InputMode }): JSX.Element {
    const highlighted = useHighlighting(value, mode === 'Template' ? 'mirascript-template' : 'mirascript');
    const { colorMode } = useColorMode();
    const { ref, inView } = useInView({ triggerOnce: true });
    const editor = inView ? (
        <Editor
            className={styles['editor']}
            value={value}
            theme={colorMode === 'dark' ? 'vs-dark' : 'vs'}
            language={mode === 'Template' ? 'mirascript-template' : 'mirascript'}
            options={{
                scrollBeyondLastLine: false,
                minimap: { enabled: false },
                fontSize: 14.4,
                lineHeight: 14.4 * 1.45,
                fontFamily: 'var(--ifm-font-family-monospace)',
                readOnly: true,
            }}
        />
    ) : null;
    return (
        <div className={styles['host']}>
            <div className={styles['editor-holder']} ref={ref}>
                {editor}
            </div>
            <pre className={styles['pre']}>
                <code dangerouslySetInnerHTML={{ __html: highlighted }} />
            </pre>
        </div>
    );
}
