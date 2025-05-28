import { editor, Uri, type IDisposable } from '@private/monaco-editor';
import { callWorker, getCompileResult } from './worker-helper.js';

const errorMessages = new Map<number, string | undefined>();
/** 获取错误消息 */
async function getErrorMessage(code: number): Promise<string | undefined> {
    if (code < 0 || code >= 65536) return undefined;
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
    const version = model.getVersionId();
    try {
        await callWorker('compile_script', model.uri);
    } catch (ex) {
        // eslint-disable-next-line no-console
        console.error(ex);
        return;
    }
    const result = getCompileResult(model.uri);
    if (!result) return;
    const { errors } = result;
    const markers: editor.IMarkerData[] = [];
    for (let i = 0; i < errors.length; i += 5) {
        const startLineNumber = errors[i];
        const startColumn = errors[i + 1];
        const endLineNumber = errors[i + 2];
        const endColumn = errors[i + 3];
        const error = errors[i + 4];
        const message = (await getErrorMessage(error)) ?? 'Unknown error';
        markers.push({
            startLineNumber,
            startColumn,
            endLineNumber,
            endColumn,
            message, //: `${message} (${startLineNumber}:${startColumn}-${endLineNumber}:${endColumn})`,
            modelVersionId: version,
            severity: 8,
            source: 'mira',
            code: {
                value: `${error}`,
                target: Uri.parse(`https://mira.cloudpss.net/code/${error}`),
            },
        });
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
