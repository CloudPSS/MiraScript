import type { InputMode } from '@mirascript/mirascript';
import { EXAMPLES } from './examples.js';
import { fromUint8Array, toUint8Array } from 'js-base64';
import { deflateSync, inflateSync } from 'fflate';

/** 主题模式 */
export type ThemeMode = 'auto' | 'light' | 'dark';

/** 状态 */
export interface State {
    /** 当前编辑器模式 */
    mode: InputMode;
    /** 当前编辑器内容 */
    source: string;
    /** 当前主题模式 */
    theme: ThemeMode;
}

const STORAGE_PREFIX = 'mirascript-playground-state-';
const hash = new URLSearchParams(location.hash.slice(1));

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

let mode: InputMode =
    (hash.get('mode') as InputMode) || (localStorage.getItem(`${STORAGE_PREFIX}mode`) as InputMode) || 'Script';
let source =
    fromHash(hash.get('source')) ||
    localStorage.getItem(`${STORAGE_PREFIX}source`) ||
    EXAMPLES[0]?.code ||
    `debug_print("Hello, World!");`;
let theme: ThemeMode = (localStorage.getItem(`${STORAGE_PREFIX}theme`) as ThemeMode) || 'auto';

hash.set('mode', mode);
hash.set('source', toHash(source));
history.replaceState({}, '', `#${hash.toString()}`);

/** 读取状态 */
export function getState(): State {
    return {
        mode,
        source,
        theme,
    };
}

/** 更新状态 */
export function setState(state: Partial<State>): void {
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
    if (state.theme != null && theme !== state.theme) {
        theme = state.theme;
        localStorage.setItem(`${STORAGE_PREFIX}theme`, theme);
    }
    history.replaceState({}, '', `#${hash.toString()}`);
}

const toast = document.createElement('div');
toast.hidden = true;
toast.className = 'toast';
document.body.append(toast);

/** 保存 URL 到剪贴板 */
async function saveUrlToClipboard(): Promise<void> {
    let ok = true;
    try {
        await navigator.clipboard.writeText(location.href);
        // Send toast
    } catch (err) {
        ok = false;
    }
    toast.hidden = false;
    toast.textContent = ok ? 'URL Has Been Copied to Clipboard' : 'Failed to Copy URL to Clipboard';
    setTimeout(() => {
        toast.hidden = true;
    }, 3000);
}

// Ctrl + S 保存 URL 到剪贴板
globalThis.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        void saveUrlToClipboard();
    }
});
