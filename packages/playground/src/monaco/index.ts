import type * as monaco from 'monaco-editor';

/**
 * monaco 运行环境
 */
const MonacoEnvironment: monaco.Environment = {
    createTrustedTypesPolicy: () => undefined,
    getWorker(_moduleId: string, label: string): Worker {
        switch (label) {
            case 'json':
                return new Worker(new URL('monaco-editor/esm/vs/language/json/json.worker.js', import.meta.url), {
                    type: 'module',
                });
            case 'css':
            case 'scss':
            case 'less':
                return new Worker(new URL('monaco-editor/esm/vs/language/css/css.worker.js', import.meta.url), {
                    type: 'module',
                });
            case 'html':
            case 'handlebars':
            case 'razor':
                return new Worker(new URL('monaco-editor/esm/vs/language/html/html.worker.js', import.meta.url), {
                    type: 'module',
                });
            case 'typescript':
            case 'javascript':
                return new Worker(new URL('monaco-editor/esm/vs/language/typescript/ts.worker.js', import.meta.url), {
                    type: 'module',
                });
            default:
                return new Worker(new URL('monaco-editor/esm/vs/editor/editor.worker.js', import.meta.url), {
                    type: 'module',
                });
        }
    },
};

Object.defineProperty(globalThis, 'MonacoEnvironment', { value: MonacoEnvironment });

import './mira/index.js';
export * from 'monaco-editor';
