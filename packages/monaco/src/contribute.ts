import { mirascriptLanguage, mirascriptDocLanguage, mirascriptTemplateLanguage } from '@mirascript/textmate/language';
import { languages } from './monaco-api.js';

export const CONTRIBUTE_IDS = Object.freeze({
    mirascript: mirascriptLanguage.name,
    mirascriptTemplate: mirascriptTemplateLanguage.name,
    mirascriptDoc: mirascriptDocLanguage.name,
    mirascriptIl: 'mirascript-il',
});

/** 注册语言 */
export function registerContribution(): void {
    languages.register({
        id: mirascriptLanguage.name,
        extensions: ['.mira'],
        aliases: mirascriptLanguage.aliases,
        mimetypes: ['text/x-mirascript'],
    });

    languages.register({
        id: mirascriptTemplateLanguage.name,
        extensions: ['.miratpl'],
        aliases: mirascriptTemplateLanguage.aliases,
        mimetypes: ['text/x-mirascript-template'],
    });

    languages.register({
        id: mirascriptDocLanguage.name,
    });

    languages.register({
        id: CONTRIBUTE_IDS.mirascriptIl,
    });
}
