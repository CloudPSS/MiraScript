import type { LanguageRegistration } from '@shikijs/types';

/** Shared TextMate language metadata. */
export type LanguageMetadata = Pick<LanguageRegistration, 'name' | 'aliases' | 'scopeName'>;

export const mirascriptLanguage = {
    name: 'mirascript',
    aliases: ['MiraScript', 'mira', 'Mira'],
    scopeName: 'source.mira',
} satisfies LanguageMetadata;

export const mirascriptTemplateLanguage = {
    name: 'mirascript-template',
    aliases: ['MiraScript-Template', 'miratpl', 'MiraTpl'],
    scopeName: 'text.miratpl',
} satisfies LanguageMetadata;

export const mirascriptDocLanguage = {
    name: 'mirascript-doc',
    scopeName: 'source.mira.doc',
} satisfies LanguageMetadata;
