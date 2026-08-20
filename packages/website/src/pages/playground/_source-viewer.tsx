import type { JSX } from 'react';
import Editor from '@site/src/components/Mira/editor';
import styles from './index.module.css';

/** 只读源码编辑器。 */
export default function SourceViewer({
    language,
    source,
    path,
    wordWrap = 'on',
}: {
    language: string;
    source: string;
    path: string;
    wordWrap?: 'on' | 'off';
}): JSX.Element {
    return (
        <Editor
            wrapperProps={{ className: styles['compiled-editor'] }}
            language={language}
            value={source}
            path={path}
            options={{
                readOnly: true,
                minimap: { enabled: false },
                wordWrap,
                colorDecorators: false,
                wrappingIndent: 'deepIndent',
            }}
        />
    );
}
