import * as monaco from '@private/monaco-editor/baseapi';
import type { editor as monacoEditor } from '@private/monaco-editor';
import { vscode } from '#loader';
import { createAdapterFactory } from './base.js';
import { toLocation } from './location.js';
import { toRange } from './range.js';
import { toUri } from './uri.js';

export const [toDiagnosticRelatedInformation, fromDiagnosticRelatedInformation] = createAdapterFactory<
    monacoEditor.IRelatedInformation,
    vscode.DiagnosticRelatedInformation
>(
    (info) => {
        return new vscode.DiagnosticRelatedInformation(toLocation({ uri: info.resource, range: info }), info.message);
    },
    (info, dri) => {
        dri.location = toLocation({ uri: info.resource, range: info });
        dri.message = info.message;
    },
);

export const [toDiagnostic, fromDiagnostic] = createAdapterFactory<monacoEditor.IMarkerData, vscode.Diagnostic>(
    (marker) => {
        return new vscode.Diagnostic(toRange(marker), marker.message);
    },
    (marker, diagnostic) => {
        diagnostic.range = toRange(marker);
        diagnostic.message = marker.message;
        switch (marker.severity) {
            case monaco.MarkerSeverity.Error:
                diagnostic.severity = vscode.DiagnosticSeverity.Error;
                break;
            case monaco.MarkerSeverity.Warning:
                diagnostic.severity = vscode.DiagnosticSeverity.Warning;
                break;
            case monaco.MarkerSeverity.Info:
                diagnostic.severity = vscode.DiagnosticSeverity.Information;
                break;
            case monaco.MarkerSeverity.Hint:
                diagnostic.severity = vscode.DiagnosticSeverity.Hint;
                break;
        }
        diagnostic.source = marker.source;
        if (typeof marker.code == 'object') {
            diagnostic.code = {
                ...marker.code,
                target: toUri(marker.code.target),
            };
        } else {
            diagnostic.code = marker.code;
        }
        diagnostic.relatedInformation = marker.relatedInformation?.map(toDiagnosticRelatedInformation);
        diagnostic.tags = marker.tags?.map((t) => {
            switch (t) {
                case monaco.MarkerTag.Deprecated:
                    return vscode.DiagnosticTag.Deprecated;
                case monaco.MarkerTag.Unnecessary:
                    return vscode.DiagnosticTag.Unnecessary;
            }
        });
    },
);
