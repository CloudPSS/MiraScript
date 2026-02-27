import pkgJson from '../../package.json' with { type: 'json' };

const { version } = pkgJson;
export const vscode = require('vscode') as typeof import('vscode');

export const { window, workspace, languages, Disposable, Uri } = vscode;
export const mira: unknown = await import(`https://esm.sh/@mirascript/mirascript@${version}/?dev`);
export const miraMonaco: unknown = await import(`https://esm.sh/@mirascript/monaco@${version}/?dev`);
export const miraMonacoLsp: unknown = await import(`https://esm.sh/@mirascript/monaco@${version}/lsp/?dev`);
