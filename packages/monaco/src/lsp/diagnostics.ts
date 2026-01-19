import { DiagnosticCode, getDiagnosticMessage } from '@mirascript/mirascript/subtle';
import { type editor, MarkerSeverity, MarkerTag, Uri, type IRange } from '../monaco-api.js';
import { Provider } from './providers/base.js';
import type { CompileResult, SourceDiagnostic } from './compile-result.js';
import { isDeprecatedGlobal } from './utils.js';

const formatMessage = (model: editor.ITextModel, template: string, $0?: string | IRange): string => {
    if (template.includes(`$0`)) {
        const replacement = typeof $0 == 'string' ? $0 : $0 ? model.getValueInRange($0) : '';
        // Replace each '$' in the captured text with '$$$$' so that after replaceAll processes
        // the string and interprets '$' as a special placeholder, we still end up with literal
        // '$$' in the final message. The quadruple dollar signs here are intentional.
        template = template.replaceAll(`$0`, replacement.replaceAll('$', '$$$$'));
    }
    return template;
};

const makeMarkerData = (
    model: editor.ITextModel,
    range: IRange,
    code: DiagnosticCode | string,
    message: string | undefined,
    severity: MarkerSeverity,
    tags?: MarkerTag[],
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
        tags,
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

/** Get code from marker */
export function getDiagnosticCode(marker: editor.IMarkerData | undefined | null): DiagnosticCode | undefined {
    if (!marker) return undefined;
    const { code } = marker;
    if (typeof code == 'object') {
        const codeName = code.value;
        if (codeName in DiagnosticCode) {
            return DiagnosticCode[codeName as keyof typeof DiagnosticCode];
        }
    } else if (typeof code == 'string') {
        if (code in DiagnosticCode) {
            return DiagnosticCode[code as keyof typeof DiagnosticCode];
        }
    }
    return undefined;
}

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
export async function makeModelMarkers(
    model: editor.ITextModel,
    result: CompileResult,
): Promise<editor.IMarkerData[] | null> {
    const { version } = result;
    if (version !== model.getVersionId()) return null;

    const errors = result.errors.map((d) => makeMarker(model, d, MarkerSeverity.Error));
    const warnings = result.warnings.map((d) => makeMarker(model, d, MarkerSeverity.Warning));
    const infos = result.infos.map((d) => makeMarker(model, d, MarkerSeverity.Info));
    const hints = result.hints.map((d) => makeMarker(model, d, MarkerSeverity.Hint));
    const markers = [...errors, ...warnings, ...infos, ...hints];
    const { globals } = result.groupedTags(model);
    if (globals.length) {
        const context = await Provider.getContext(model);
        for (const g of globals) {
            const { name } = g;
            if (!context.has(name)) {
                const template = getDiagnosticMessage(DiagnosticCode.GlobalVariableNotDeclared);
                if (!template) continue;
                const message = formatMessage(model, template, name);
                for (const ref of g.references) {
                    markers.push(
                        makeMarkerData(
                            model,
                            ref.range,
                            DiagnosticCode.GlobalVariableNotDeclared,
                            message,
                            MarkerSeverity.Warning,
                        ),
                    );
                }
                continue;
            }
            const deprecated = isDeprecatedGlobal(context, name);
            if (deprecated) {
                const replacement = deprecated.use;
                if (replacement) {
                    const template = getDiagnosticMessage(deprecated.message);
                    if (!template) continue;
                    const message = formatMessage(model, template, replacement);
                    for (const ref of g.references) {
                        markers.push(
                            makeMarkerData(model, ref.range, deprecated.message, message, MarkerSeverity.Hint, [
                                MarkerTag.Deprecated,
                            ]),
                        );
                    }
                } else {
                    for (const ref of g.references) {
                        markers.push(
                            makeMarkerData(model, ref.range, deprecated.message, undefined, MarkerSeverity.Hint, [
                                MarkerTag.Deprecated,
                            ]),
                        );
                    }
                }
            }
        }
    }
    return markers;
}
