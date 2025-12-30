import { getState, setState, type ThemeMode } from './state-manager.js';

const systemTheme = matchMedia('(prefers-color-scheme: dark)');

const callbacks = new Set<(theme: Exclude<ThemeMode, 'auto'>) => void>();

export const currentTheme = (): Exclude<ThemeMode, 'auto'> => {
    const { theme } = getState();
    if (theme === 'auto') {
        return systemTheme.matches ? 'dark' : 'light';
    }
    return theme;
};

/** 监听主题变化 */
export function onThemeChange(callback: (theme: Exclude<ThemeMode, 'auto'>) => void): (() => void) | undefined {
    if (callbacks.has(callback)) return undefined;
    callbacks.add(callback);
    callback(currentTheme());
    return () => callbacks.delete(callback);
}

/** 更新主题 */
function updateTheme() {
    const { theme } = getState();
    document.documentElement.dataset['theme'] = theme;
    const calculated = currentTheme();
    for (const cb of callbacks) {
        cb(calculated);
    }
}
systemTheme.addEventListener('change', () => {
    const { theme } = getState();
    if (theme === 'auto') {
        updateTheme();
    }
});

// 应用初始主题
updateTheme();

/** 初始化主题选择器 */
setTimeout(() => {
    const { theme } = getState();
    const elThemeSelect = document.querySelector<HTMLSelectElement>('#theme-select')!;
    elThemeSelect.value = theme;

    elThemeSelect.addEventListener('change', () => {
        const newTheme = elThemeSelect.value as ThemeMode;
        setState({ theme: newTheme });
        updateTheme();
    });
});
