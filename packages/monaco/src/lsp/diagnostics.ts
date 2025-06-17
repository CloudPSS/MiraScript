import { editor, MarkerSeverity, MarkerTag, Uri } from '../monaco-api.js';
// import { Provider } from './base.js';
import { DiagnosticCode, getDiagnosticMessage } from 'mirascript';
import type { CompileResult, SourceDiagnostic } from './compile-result.js';

// const onlyVisible = false;
// const validateDelay = 500;
// const listeners = new Map<string, IDisposable>();

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
// /** 进行测试 */
// async function validate(model: editor.ITextModel): Promise<void> {
//     if (onlyVisible && !model.isAttachedToEditor()) {
//         editor.setModelMarkers(model, 'mirascript', []);
//         return;
//     }
//     const result = await Provider.getCompileResult(model);
//     if (!result) return;
//     setMarkers(model, result);
// }

/** 设置标记 */
export function setMarkers(model: editor.ITextModel, result: CompileResult): void {
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

// /** 添加模型 */
// function onModelAdd(model: editor.ITextModel): void {
//     if (model.getLanguageId() !== 'mirascript') return;
//     let handle: ReturnType<typeof setTimeout>;
//     const change = model.onDidChangeContent(() => {
//         clearTimeout(handle);
//         handle = setTimeout(() => void validate(model), validateDelay);
//     });
//     const visibility = model.onDidChangeAttached(() => void validate(model));
//     listeners.set(model.uri.toString(), {
//         dispose() {
//             change.dispose();
//             visibility.dispose();
//             clearTimeout(handle);
//         },
//     });
//     void validate(model);
// }

// /** 移除模型 */
// function onModelRemoved(model: editor.ITextModel): void {
//     editor.setModelMarkers(model, 'mirascript', []);
//     const key = model.uri.toString();
//     const disposable = listeners.get(key);
//     if (disposable) {
//         disposable.dispose();
//         listeners.delete(key);
//     }
// }

// const disposables: IDisposable[] = [];

// disposables.push(
//     editor.onDidCreateModel((model) => onModelAdd(model)),
//     editor.onWillDisposeModel((model) => onModelRemoved(model)),
//     editor.onDidChangeModelLanguage((event) => {
//         onModelRemoved(event.model);
//         onModelAdd(event.model);
//     }),
//     {
//         dispose() {
//             for (const model of editor.getModels()) {
//                 onModelRemoved(model);
//             }
//             for (const disposable of listeners.values()) {
//                 disposable.dispose();
//             }
//             listeners.clear();
//         },
//     },
// );

// for (const model of editor.getModels()) {
//     onModelAdd(model);
// }

// /** 重新计算所有 */
// export function recomputeDiagnostics(): void {
//     for (const model of editor.getModels()) {
//         onModelRemoved(model);
//         onModelAdd(model);
//     }
// }
