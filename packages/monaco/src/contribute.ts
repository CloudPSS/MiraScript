import { mirascriptLanguage, mirascriptDocLanguage, mirascriptTemplateLanguage } from '@mirascript/textmate/language';
import { languages } from './monaco-api.js';

export const CONTRIBUTE_IDS = [mirascriptLanguage.name, mirascriptTemplateLanguage.name, mirascriptDocLanguage.name];

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
}
