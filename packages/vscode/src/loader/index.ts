export * as vscode from 'vscode';
export { window, workspace, languages, Disposable, Uri } from 'vscode';
export * as mira from '@mirascript/mirascript';
export * as miraMonaco from '@mirascript/monaco';
export * as miraMonacoLsp from '@mirascript/monaco/lsp';

import { pathToFileURL } from 'node:url';
import { cosmiconfig, type CosmiconfigResult, type Loader } from 'cosmiconfig';
import type { Uri } from 'vscode';

const jsLoader: Loader = async (filepath, content) => {
    const url = pathToFileURL(filepath);
    url.search = content;
    const mod = (await import(url.href)) as { default: unknown };
    return mod.default ?? mod;
};
const explorer = cosmiconfig('mira', {
    cache: false,
    searchStrategy: 'project',
    loaders: {
        '.js': jsLoader,
        '.cjs': jsLoader,
        '.mjs': jsLoader,
        '.ts': jsLoader,
        '.mts': jsLoader,
        '.cts': jsLoader,
    },
});

export const searchConfig = async (uri: Uri): Promise<CosmiconfigResult> => {
    return explorer.search(uri.fsPath);
};

export const loadConfig = async (uri: Uri): Promise<CosmiconfigResult> => {
    return explorer.load(uri.fsPath);
};
