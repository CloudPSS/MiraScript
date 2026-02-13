import { useEffect, useState, type JSX } from 'react';
import { useMonaco } from './monaco';
import { useColorMode } from '@docusaurus/theme-common';

/** 语法高亮 */
export default function Highlight({
    code,
    language,
    ...props
}: {
    code: string;
    language?: 'mirascript' | 'mirascript-template' | 'mirascript-doc';
}): JSX.Element {
    const { colorMode } = useColorMode();
    const monaco = useMonaco();
    const [html, setHtml] = useState<string>('');
    useEffect(() => {
        if (!monaco) return;
        void monaco.editor.colorize(code, language ?? 'mirascript', {}).then((colored) => {
            if (!code.endsWith('\n') && colored.endsWith('<br/>')) {
                colored = colored.slice(0, -5);
            }
            setHtml(colored);
        });
    }, [monaco, code, language, colorMode]);
    if (!html) {
        return <span {...props}>{code}</span>;
    }
    return <span dangerouslySetInnerHTML={{ __html: html }} {...props} />;
}
