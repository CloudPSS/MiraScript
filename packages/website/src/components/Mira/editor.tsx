import { useMemo, useRef, type JSX } from 'react';
import { useColorMode } from '@docusaurus/theme-common';
import { Editor as InnerEditor, type EditorProps } from '@monaco-editor/react';
import { useMonaco } from './monaco';
import type { IDisposable } from '@private/monaco-editor';

/** Monaco editor */
export default function Editor(props: EditorProps): JSX.Element {
    const { colorMode } = useColorMode();
    const monaco = useMonaco();
    const elRef = useRef<HTMLElement>(null);
    const overlayRef = useRef<HTMLDivElement & IDisposable>(null);
    const overlay = useMemo(() => {
        if (monaco && elRef.current) {
            return monaco.utils.createOverflowWidgetsDomNode(elRef.current);
        }
        return null;
    }, [monaco, elRef]);
    props = { ...props };
    props.theme ??= colorMode === 'dark' ? 'vs-dark' : 'vs-light';
    props.options = {
        fontFamily: 'var(--ifm-font-family-monospace)',
        fontLigatures: true,
        automaticLayout: true,
        tabSize: 2,
        'semanticHighlighting.enabled': true,
        formatOnType: true,
        formatOnPaste: true,
        ...props.options,
    };
    props.wrapperProps = {
        ...props.wrapperProps,
        ref: elRef,
    };
    const { onMount } = props;
    props.onMount = (editor, monaco: typeof import('@private/monaco-editor')) => {
        editor.onDidDispose(() => {
            overlayRef.current?.dispose();
            overlayRef.current = null;
        });
        onMount?.(editor, monaco);
    };
    if (overlay) {
        props.options.useShadowDOM = true;
        props.options.overflowWidgetsDomNode = overlay;
        if (overlayRef.current !== overlay) {
            overlayRef.current?.dispose();
            overlayRef.current = overlay;
        }
    }
    return <InnerEditor {...props} />;
}
