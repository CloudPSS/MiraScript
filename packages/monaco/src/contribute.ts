import { languages } from './monaco-api.js';

/** 注册语言 */
export function registerContribution(): void {
    languages.register({
        id: 'mirascript',
        extensions: ['.mira'],
        aliases: ['MiraScript', 'mira', 'Mira'],
        mimetypes: ['text/x-mirascript'],
    });

    languages.register({
        id: 'mirascript-template',
        extensions: ['.miratpl'],
        aliases: ['MiraScript-Template', 'miratpl', 'MiraTpl'],
        mimetypes: ['text/x-mirascript-template'],
    });

    languages.register({
        id: 'mirascript-doc',
        extensions: [],
        aliases: [],
        mimetypes: ['text/x-mirascript-doc'],
    });
}
