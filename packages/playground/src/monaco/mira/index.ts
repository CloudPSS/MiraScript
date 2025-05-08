import { languages } from 'monaco-editor';

languages.register({
    id: 'mirascript',
    extensions: ['.mira'],
    aliases: ['MiraScript', 'mirascript', 'mira'],
    mimetypes: ['text/x-mirascript'],
});

languages.onLanguage('mirascript', () => void import('./loader.js'));
