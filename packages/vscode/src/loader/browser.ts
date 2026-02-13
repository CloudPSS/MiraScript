import pkgJson from '../../package.json' with { type: 'json' };

const { version } = pkgJson;
export const vscode = require('vscode') as typeof import('vscode');

export const { window, workspace, languages, Disposable, Uri } = vscode;
export const mira: unknown = await import(`https://esm.sh/@mirascript/mirascript@${version}/?dev`);
export const miraMonaco: unknown = await import(`https://esm.sh/@mirascript/monaco@${version}/?dev`);
export const miraMonacoLsp: unknown = await import(`https://esm.sh/@mirascript/monaco@${version}/lsp/?dev`);

import type { Uri as URI } from 'vscode';
import type { CosmiconfigResult } from 'cosmiconfig';
import { load as parseYaml } from 'js-yaml';
const { fs } = vscode.workspace;

export const searchConfig = async (uri: URI): Promise<CosmiconfigResult> => {
    const configs = await vscode.workspace.findFiles(
        '**/.mira{rc,.config}{,.json,.yaml,.yml,.js,.cjs,.mjs,.ts,.mts,.cts}',
        '**/node_modules/**',
    );
    if (!configs.length) {
        return null;
    }
    for (const config of configs) {
        if (uri.path.startsWith(config.path.replace(/\/?\.mira(rc|\.config)?\..*$/, ''))) {
            return loadConfig(config);
        }
    }
    return null;
};
export const loadConfig = async (uri: URI): Promise<CosmiconfigResult> => {
    const content = await fs.readFile(uri);
    const str = new TextDecoder().decode(content);
    let parsed: unknown;
    if (uri.path.endsWith('.json')) {
        parsed = JSON.parse(str);
    } else if (
        uri.path.endsWith('.js') ||
        uri.path.endsWith('.cjs') ||
        uri.path.endsWith('.mjs') ||
        uri.path.endsWith('.ts') ||
        uri.path.endsWith('.mts') ||
        uri.path.endsWith('.cts')
    ) {
        const mod = (await import(`data:application/javascript,${encodeURIComponent(str)}`)) as { default: unknown };
        parsed = mod.default ?? mod;
    } else {
        parsed = parseYaml(str);
    }
    return {
        filepath: uri as unknown as string,
        config: parsed,
    } as CosmiconfigResult;
};
