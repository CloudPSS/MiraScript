import { languages } from 'monaco-editor';

languages.registerCodeActionProvider('mirascript', {
    provideCodeActions(model, range, context, token) {
        return null;
    },
});
