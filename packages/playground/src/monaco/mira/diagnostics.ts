import { editor, MarkerSeverity, Uri, type IDisposable } from '@private/monaco-editor';
import { callWorker, Provider } from './worker-helper.js';
import { DiagnosticCode } from 'mira-wasm';

const errorMessages = new Map<DiagnosticCode, string | undefined>();
/** 获取错误消息 */
async function getErrorMessage(code: DiagnosticCode): Promise<string | undefined> {
    if (errorMessages.has(code)) return errorMessages.get(code);
    const message = await callWorker('get_error_message', code);
    errorMessages.set(code, message);
    return message;
}

const onlyVisible = false;
const validateDelay = 500;
const listeners = new Map<string, IDisposable>();

/** 进行测试 */
async function validate(model: editor.ITextModel): Promise<void> {
    if (onlyVisible && !model.isAttachedToEditor()) {
        editor.setModelMarkers(model, 'mirascript', []);
        return;
    }
    const result = await Provider.getCompileResult(model);
    if (!result) return;
    const { diagnostics, version } = result;
    const markers: editor.IMarkerData[] = [];
    for (let i = 0; i < diagnostics.length; i++) {
        const diagnostic = diagnostics[i]!;
        const { startLineNumber, startColumn, endLineNumber, endColumn, code } = diagnostic;
        const message = (await getErrorMessage(code)) ?? 'Unknown error';
        let severity: MarkerSeverity;
        if (code > DiagnosticCode.ErrorStart && code < DiagnosticCode.ErrorEnd) {
            severity = MarkerSeverity.Error;
        } else if (code > DiagnosticCode.WarningStart && code < DiagnosticCode.WarningEnd) {
            severity = MarkerSeverity.Warning;
        } else if (code > DiagnosticCode.InfoStart && code < DiagnosticCode.InfoEnd) {
            severity = MarkerSeverity.Info;
        } else if (code > DiagnosticCode.HintStart && code < DiagnosticCode.HintEnd) {
            severity = MarkerSeverity.Hint;
        } else {
            continue;
        }
        const marker: editor.IMarkerData = {
            startLineNumber,
            startColumn,
            endLineNumber,
            endColumn,
            message, //: `${message} (${startLineNumber}:${startColumn}-${endLineNumber}:${endColumn})`,
            modelVersionId: version,
            severity,
            source: 'mira',
            code: {
                value: `${code}`,
                target: Uri.parse(`https://mira.cloudpss.net/code/${code}`),
            },
        };
        while (i + 1 < diagnostics.length) {
            const next = diagnostics[i + 1]!;
            if (next.code <= DiagnosticCode.ReferenceStart || next.code >= DiagnosticCode.ReferenceEnd) {
                break;
            }
            i++;
            marker.relatedInformation ??= [];
            marker.relatedInformation.push({
                message: (await getErrorMessage(next.code)) ?? 'Reference',
                resource: model.uri,
                startLineNumber: next.startLineNumber,
                startColumn: next.startColumn,
                endLineNumber: next.endLineNumber,
                endColumn: next.endColumn,
            });
        }
        markers.push(marker);
    }
    editor.setModelMarkers(model, 'mirascript', markers);
}

/** 添加模型 */
function onModelAdd(model: editor.ITextModel): void {
    if (model.getLanguageId() !== 'mirascript') return;
    let handle: ReturnType<typeof setTimeout>;
    const change = model.onDidChangeContent(() => {
        clearTimeout(handle);
        handle = setTimeout(() => void validate(model), validateDelay);
    });
    const visibility = model.onDidChangeAttached(() => void validate(model));
    listeners.set(model.uri.toString(), {
        dispose() {
            change.dispose();
            visibility.dispose();
            clearTimeout(handle);
        },
    });
    void validate(model);
}

/** 移除模型 */
function onModelRemoved(model: editor.ITextModel): void {
    editor.setModelMarkers(model, 'mirascript', []);
    const key = model.uri.toString();
    const disposable = listeners.get(key);
    if (disposable) {
        disposable.dispose();
        listeners.delete(key);
    }
}

const disposables: IDisposable[] = [];

disposables.push(
    editor.onDidCreateModel((model) => onModelAdd(model)),
    editor.onWillDisposeModel((model) => onModelRemoved(model)),
    editor.onDidChangeModelLanguage((event) => {
        onModelRemoved(event.model);
        onModelAdd(event.model);
    }),
    {
        dispose() {
            for (const model of editor.getModels()) {
                onModelRemoved(model);
            }
            for (const disposable of listeners.values()) {
                disposable.dispose();
            }
            listeners.clear();
        },
    },
);

for (const model of editor.getModels()) {
    onModelAdd(model);
}

/** 重新计算所有 */
export function recomputeDiagnostics(): void {
    for (const model of editor.getModels()) {
        onModelRemoved(model);
        onModelAdd(model);
    }
}
