import { DiagnosticCode, getDiagnosticMessage } from '@mirascript/mirascript/subtle';
import { editor, MarkerSeverity, MarkerTag, Uri } from '../monaco-api.js';
import type { CompileResult, SourceDiagnostic } from './compile-result.js';

const makeMarker = (
    model: editor.ITextModel,
    modelVersionId: number,
    diagnostic: SourceDiagnostic,
    severity: MarkerSeverity,
): editor.IMarkerData => {
    const { range, code } = diagnostic;
    const { startLineNumber, startColumn, endLineNumber, endColumn } = range;
    let unnecessary = false;
    const deprecated = false;
    if (code === DiagnosticCode.UnusedLocalVariable || code === DiagnosticCode.UnusedLocalFunction) {
        unnecessary = true;
        severity = MarkerSeverity.Hint;
    }
    let message = getDiagnosticMessage(code) ?? 'Unknown error';
    if (message.includes(`$0`)) {
        message = message.replaceAll(`$0`, model.getValueInRange(range));
    }
    const marker: editor.IMarkerData = {
        startLineNumber,
        startColumn,
        endLineNumber,
        endColumn,
        message,
        severity,
        modelVersionId,
        source: 'MiraScript',
    };
    const codeName = DiagnosticCode[code];
    if (codeName) {
        marker.code = {
            value: codeName,
            target: Uri.parse(`https://mira.cloudpss.net/code/${codeName}`),
        };
    } else {
        marker.code = `${code}`;
    }
    if (unnecessary) {
        marker.tags ??= [];
        marker.tags.push(MarkerTag.Unnecessary);
    }
    if (deprecated) {
        marker.tags ??= [];
        marker.tags.push(MarkerTag.Deprecated);
    }
    if (diagnostic.references.length) {
        marker.relatedInformation = [];
        for (const ref of diagnostic.references) {
            const { range, code } = ref;
            const { startLineNumber, startColumn, endLineNumber, endColumn } = range;
            const message = getDiagnosticMessage(code) ?? '...here';
            marker.relatedInformation.push({
                message,
                resource: model.uri,
                startLineNumber,
                startColumn,
                endLineNumber,
                endColumn,
            });
        }
    }
    return marker;
};

/** 设置标记 */
export function setMarkers(model: editor.ITextModel, result: CompileResult): void {
    if (typeof editor?.setModelMarkers != 'function') {
        return;
    }
    const { version } = result;
    if (version !== model.getVersionId()) {
        return;
    }
    const errors = result.errors.map((d) => makeMarker(model, version, d, MarkerSeverity.Error));
    const warnings = result.warnings.map((d) => makeMarker(model, version, d, MarkerSeverity.Warning));
    const infos = result.infos.map((d) => makeMarker(model, version, d, MarkerSeverity.Info));
    const hints = result.hints.map((d) => makeMarker(model, version, d, MarkerSeverity.Hint));
    const markers = [...errors, ...warnings, ...infos, ...hints];
    editor.setModelMarkers(model, 'mirascript', markers);
}
