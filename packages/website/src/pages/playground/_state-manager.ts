import useIsBrowser from '@docusaurus/useIsBrowser';
import { useEffect, useState } from 'react';
import type { InputMode } from '@mirascript/mirascript';
import { EXAMPLES } from './_examples';
import { fromUint8Array, toUint8Array } from 'js-base64';
import { deflateSync, inflateSync } from 'fflate';

/** 状态 */
export interface State {
    /** 当前编辑器模式 */
    mode: InputMode;
    /** 当前编辑器内容 */
    source: string;
}

let getState: () => State, setState: (state: Partial<State>) => void;

const defaultCode = (await EXAMPLES[0]?.code()) || `debug_print("Hello, World!");`;
const defaultMode: InputMode = 'Script';

/** 保存 URL 到剪贴板 */
function saveUrlToClipboard(e: KeyboardEvent): void {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        void navigator.clipboard.writeText(location.href);
    }
}

/** 初始化状态管理器 */
function initStateManager(canUseDOM: boolean): void {
    const STORAGE_PREFIX = 'mirascript-playground-state-';
    const hash = new URLSearchParams(canUseDOM ? location.hash.slice(1) : '');

    const fromHash = (value: string | null): string | undefined => {
        if (!value) return undefined;
        try {
            const decoded = toUint8Array(value);
            const uncompressed = inflateSync(decoded);
            return new TextDecoder().decode(uncompressed);
        } catch {
            return undefined;
        }
    };

    const toHash = (value: string | undefined): string => {
        if (!value) return '';
        try {
            const encoded = new TextEncoder().encode(value);
            const compressed = deflateSync(encoded, { level: 9 });
            return fromUint8Array(compressed, true);
        } catch {
            return '';
        }
    };

    let mode: InputMode = canUseDOM
        ? (hash.get('mode') as InputMode) || (localStorage.getItem(`${STORAGE_PREFIX}mode`) as InputMode) || defaultMode
        : defaultMode;
    let source = canUseDOM
        ? fromHash(hash.get('source')) || localStorage.getItem(`${STORAGE_PREFIX}source`) || defaultCode
        : defaultCode;

    hash.set('mode', mode);
    hash.set('source', toHash(source));

    /** 读取状态 */
    getState = (): State => {
        return {
            mode,
            source,
        };
    };

    /** 更新状态 */
    setState = (state: Partial<State>): void => {
        if (state.mode != null && mode !== state.mode) {
            mode = state.mode;
            localStorage.setItem(`${STORAGE_PREFIX}mode`, mode);
            hash.set('mode', mode);
        }
        if (state.source != null && source !== state.source) {
            source = state.source;
            localStorage.setItem(`${STORAGE_PREFIX}source`, source);
            hash.set('source', toHash(source));
        }
        history.replaceState({}, '', `#${hash.toString()}`);
    };
    if (canUseDOM) {
        // Ctrl + S 保存 URL 到剪贴板
        globalThis.addEventListener('keydown', saveUrlToClipboard);
    }
}

/** 演练场上下文 */
export function usePlaygroundState(): [State, (state: Partial<State>) => void] {
    const isBrowser = useIsBrowser();
    const [currentState, setCurrentState] = useState<State>({ mode: defaultMode, source: defaultCode });
    useEffect(() => {
        initStateManager(isBrowser);
        setCurrentState(getState());
    }, [isBrowser]);

    const setStateWrapper = (state: Partial<State>): void => {
        setState(state);
        setCurrentState(getState());
    };

    return [currentState, setStateWrapper];
}
