import { DiagnosticCode, getDiagnosticMessage } from '@mirascript/mirascript/subtle';
import { editor, MarkerSeverity, MarkerTag, Uri, type IRange } from '../monaco-api.js';
import type { CompileResult, SourceDiagnostic } from './compile-result.js';
import { getContext } from './providers/base.js';

const formatMessage = (model: editor.ITextModel, template: string, $0?: string | IRange): string => {
    if (template.includes(`$0`)) {
        const replacement = typeof $0 == 'string' ? $0 : $0 ? model.getValueInRange($0) : '';
        template = template.replaceAll(`$0`, replacement);
    }
    return template;
};

const makeMarkerData = (
    model: editor.ITextModel,
    range: IRange,
    code: DiagnosticCode | string,
    message: string | undefined,
    severity: MarkerSeverity,
): editor.IMarkerData => {
    const { startLineNumber, startColumn, endLineNumber, endColumn } = range;
    if (!message) {
        if (typeof code != 'number') {
            message = '';
        } else {
            const template = getDiagnosticMessage(code);
            message = template ? formatMessage(model, template, range) : 'Unknown diagnostic';
        }
    }
    const marker: editor.IMarkerData = {
        startLineNumber,
        startColumn,
        endLineNumber,
        endColumn,
        message,
        severity,
        modelVersionId: model.getVersionId(),
        source: 'MiraScript',
    };
    const codeName = typeof code == 'number' ? DiagnosticCode[code] : undefined;
    if (codeName) {
        marker.code = {
            value: codeName,
            target: Uri.parse(`https://mira.cloudpss.net/code/${codeName}`),
        };
    } else {
        marker.code = `${code}`;
    }
    return marker;
};

const makeMarker = (
    model: editor.ITextModel,
    diagnostic: SourceDiagnostic,
    severity: MarkerSeverity,
): editor.IMarkerData => {
    const { range, code } = diagnostic;
    let unnecessary = false;
    const deprecated = false;
    if (code === DiagnosticCode.UnusedLocalVariable || code === DiagnosticCode.UnusedLocalFunction) {
        unnecessary = true;
        severity = MarkerSeverity.Hint;
    }
    const marker = makeMarkerData(model, range, code, undefined, severity);
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
export async function setMarkers(model: editor.ITextModel, result: CompileResult): Promise<void> {
    const setModelMarkers = editor?.setModelMarkers;
    if (typeof setModelMarkers != 'function') return;

    const { version } = result;
    if (version !== model.getVersionId()) return;

    const errors = result.errors.map((d) => makeMarker(model, d, MarkerSeverity.Error));
    const warnings = result.warnings.map((d) => makeMarker(model, d, MarkerSeverity.Warning));
    const infos = result.infos.map((d) => makeMarker(model, d, MarkerSeverity.Info));
    const hints = result.hints.map((d) => makeMarker(model, d, MarkerSeverity.Hint));
    const markers = [...errors, ...warnings, ...infos, ...hints];
    const { globals } = result.groupedTags(model);
    if (globals.length) {
        const context = await getContext(model);
        for (const g of globals) {
            const { name } = g;
            if (context.has(name)) continue;
            const template = getDiagnosticMessage(DiagnosticCode.GlobalVariableNotDeclared);
            if (!template) continue;
            const message = formatMessage(model, template, name);
            markers.push(
                makeMarkerData(
                    model,
                    g.references[0]!.range,
                    DiagnosticCode.GlobalVariableNotDeclared,
                    message,
                    MarkerSeverity.Warning,
                ),
            );
        }
    }
    setModelMarkers(model, 'mirascript', markers);
}
