import type { languages as monacoLanguages } from '@private/monaco-editor';
import { vscode } from '#loader';
import { createAdapterFactory } from './base.js';
import { toCommand } from './command.js';
import { toLocation } from './location.js';
import { toMarkdownString } from './markdown-string.js';
import { toPosition } from './position.js';
import { toTextEdit } from './text-edit.js';

export const [toInlayHintLabelPart, fromInlayHintLabelPart] = createAdapterFactory<
    monacoLanguages.InlayHintLabelPart,
    vscode.InlayHintLabelPart
>(
    (part) => new vscode.InlayHintLabelPart(part.label),
    (part, lp) => {
        lp.value = part.label;
        lp.tooltip = toMarkdownString(part.tooltip);
        lp.location = toLocation(part.location);
        lp.command = toCommand(part.command);
    },
);

export const [toInlayHint, fromInlayHint] = createAdapterFactory<monacoLanguages.InlayHint, vscode.InlayHint>(
    (item) =>
        new vscode.InlayHint(
            toPosition(item.position),
            typeof item.label === 'string' ? item.label : item.label.map(toInlayHintLabelPart),
        ),
    (item, h) => {
        h.label = typeof item.label === 'string' ? item.label : item.label.map(toInlayHintLabelPart);
        h.position = toPosition(item.position);
        h.kind = item.kind;
        h.paddingLeft = item.paddingLeft;
        h.paddingRight = item.paddingRight;
        h.tooltip = toMarkdownString(item.tooltip);
        h.textEdits = item.textEdits?.map(toTextEdit);
    },
);
