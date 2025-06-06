import { languages } from '@private/monaco-editor';

languages.register({
    id: 'mirascript',
    extensions: ['.mira'],
    aliases: ['MiraScript', 'mirascript', 'mira'],
    mimetypes: ['text/x-mirascript'],
});

languages.onLanguage('mirascript', () => void import('./loader.js'));
languages.onLanguageEncountered('mirascript', () => void import('./loader-small.js'));
