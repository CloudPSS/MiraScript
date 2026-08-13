import type { languages as monacoLanguages } from '@private/monaco-editor';
import { vscode } from '#loader';
import { createAdapterFactory } from './base.js';
import { toRange } from './range.js';

export const [toDocumentSymbol, fromDocumentSymbol] = createAdapterFactory<
    monacoLanguages.DocumentSymbol,
    vscode.DocumentSymbol
>(
    (symbol) =>
        new vscode.DocumentSymbol(
            symbol.name,
            symbol.detail,
            symbol.kind,
            toRange(symbol.range),
            toRange(symbol.selectionRange),
        ),
    (symbol, ds) => {
        ds.name = symbol.name;
        ds.detail = symbol.detail;
        ds.kind = symbol.kind;
        ds.range = toRange(symbol.range);
        ds.selectionRange = toRange(symbol.selectionRange);
        ds.children = symbol.children?.map(toDocumentSymbol) ?? [];
    },
);
