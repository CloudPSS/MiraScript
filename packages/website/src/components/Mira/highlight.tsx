import { useEffect, useState, type JSX } from 'react';
import { useMonaco } from './monaco';

/** 语法高亮 */
export function Highlight({ code, language, ...props }: { code: string; language?: 'mirascript' | 'mirascript-template' }): JSX.Element {
    const monaco = useMonaco();
    const [html, setHtml] = useState<string>('');
    useEffect(() => {
        if (!monaco) return;
        void monaco.editor.colorize(code, language ?? 'mirascript', {}).then((colored) => {
            if (colored.endsWith('<br/>')) {
                colored = colored.slice(0, -5);
            }
            setHtml(colored);
        });
    }, [monaco, code, language]);
    if (!html) {
        return <span {...props}>{code}</span>;
    }
    return <span dangerouslySetInnerHTML={{ __html: html }} {...props} />;
}
