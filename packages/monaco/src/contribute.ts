import { languages } from './monaco-api.js';

/** 注册语言 */
export function registerContribution(): void {
    languages.register({
        id: 'mirascript',
        extensions: ['.mira'],
        aliases: ['MiraScript', 'mirascript', 'mira'],
        mimetypes: ['text/x-mirascript'],
    });

    languages.register({
        id: 'mirascript-template',
        extensions: ['.miratpl'],
        aliases: ['MiraScriptTemplate', 'mirascript-template', 'miratpl'],
        mimetypes: ['text/x-mirascript-template'],
    });

    languages.register({
        id: 'mirascript-doc',
        extensions: [],
        aliases: [],
        mimetypes: ['text/x-mirascript-doc'],
    });
}
