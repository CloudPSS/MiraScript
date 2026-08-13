import type { languages as monacoLanguages } from '@private/monaco-editor';
import { vscode } from '#loader';
import { createAdapterFactory } from './base.js';
import { toMarkdownString } from './markdown-string.js';

export const [toParameterInformation, fromParameterInformation] = createAdapterFactory<
    monacoLanguages.ParameterInformation,
    vscode.ParameterInformation
>(
    (param) => new vscode.ParameterInformation(param.label),
    (param, pi) => {
        pi.label = param.label;
        pi.documentation = toMarkdownString(param.documentation);
    },
);

export const [toSignatureInformation, fromSignatureInformation] = createAdapterFactory<
    monacoLanguages.SignatureInformation,
    vscode.SignatureInformation
>(
    (sig) => new vscode.SignatureInformation(sig.label),
    (sig, si) => {
        si.label = sig.label;
        si.documentation = toMarkdownString(sig.documentation);
        si.parameters = sig.parameters?.map(toParameterInformation);
    },
);

export const [toSignatureHelp, fromSignatureHelp] = createAdapterFactory<
    monacoLanguages.SignatureHelp,
    vscode.SignatureHelp
>(
    () => new vscode.SignatureHelp(),
    (sh, s) => {
        s.signatures = sh.signatures.map(toSignatureInformation);
        s.activeParameter = sh.activeParameter;
        s.activeSignature = sh.activeSignature;
    },
);
