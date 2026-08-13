import type { languages as monacoLanguages } from '@private/monaco-editor';
import { vscode } from '#loader';
import { createAdapterFactory } from './base.js';
import { toCommand } from './command.js';
import { toRange } from './range.js';

export const [toCodeLens, fromCodeLens] = createAdapterFactory<monacoLanguages.CodeLens, vscode.CodeLens>(
    (lens) => new vscode.CodeLens(toRange(lens.range)),
    (lens, cl) => {
        cl.range = toRange(lens.range);
        cl.command = toCommand(lens.command);
    },
);
