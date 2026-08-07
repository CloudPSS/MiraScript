import type { JSX } from 'react';
import Editor from '@site/src/components/Mira/editor';
import styles from './index.module.css';

/** 只读源码编辑器。 */
export default function SourceViewer({ language, source, path }: { language: string; source: string; path: string }): JSX.Element {
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
