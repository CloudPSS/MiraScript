import { languages } from '@private/monaco-editor';

languages.registerCodeActionProvider('mirascript', {
    provideCodeActions(model, range, context, token) {
        return null;
    },
});
