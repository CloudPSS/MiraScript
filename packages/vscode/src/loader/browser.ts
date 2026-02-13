export const vscode = require('vscode') as typeof import('vscode');

export const { window, workspace, languages, Disposable, Uri } = vscode;
export const mira: unknown = await import('https://esm.sh/@mirascript/mirascript');
export const miraMonaco: unknown = await import('https://esm.sh/@mirascript/monaco');
export const miraMonacoLsp: unknown = await import('https://esm.sh/@mirascript/monaco/lsp');

export const explorer = {
    load(): null {
        return null;
    },
    search(): null {
        return null;
    },
};
