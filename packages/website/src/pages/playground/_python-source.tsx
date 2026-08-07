import { useEffect, useRef, useState, type JSX } from 'react';
import type { Results } from '@site/src/components/Mira/runner';
import type { PythonSourceRequest, PythonSourceResponse } from './_python-source-protocol';
import SourceViewer from './_source-viewer';
import styles from './index.module.css';

let worker: Worker | undefined;
let requestId = 0;
const pending = new Map<
    number,
    {
        /** 完成请求。 */
        resolve(source: string): void;
        /** 拒绝请求。 */
        reject(error: Error): void;
    }
>();

/** 获取 Python 代码生成 worker。 */
function getWorker(): Worker {
    if (worker) return worker;
    worker = new Worker(new URL('./_python-source-worker.ts', import.meta.url), { type: 'module', name: 'pyodide' });
    worker.addEventListener('message', (event: MessageEvent<PythonSourceResponse>) => {
        const response = event.data;
        const request = pending.get(response.id);
        if (!request) return;
        pending.delete(response.id);
        if (response.error == null) request.resolve(response.source);
        else request.reject(new Error(response.error));
    });
    worker.addEventListener('error', (event) => {
        const error = new Error(event.message || 'Python source worker failed.');
        for (const request of pending.values()) request.reject(error);
        pending.clear();
        worker?.terminate();
        worker = undefined;
    });
    return worker;
}

/** 使用 Pyodide 生成 artifact 对应的 Python 源代码。 */
async function generatePythonSource(artifact: Results): Promise<string> {
    const id = ++requestId;
    const request: PythonSourceRequest = {
        id,
        source: artifact.source,
        mode: artifact.mode,
        fileName: artifact.fileName,
    };
    return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject });
        getWorker().postMessage(request);
    });
}

/** Python 源代码页签状态。 */
type PythonState =
    | { status: 'idle'; artifact?: never }
    | { status: 'loading'; artifact: Results }
    | { status: 'ready'; artifact: Results; source: string }
    | { status: 'error'; artifact: Results; message: string };

/** 显示 Python 源码 */
export default function PythonSourceViewer({ artifact }: { artifact: Results | null }): JSX.Element {
    const currentArtifact = useRef(artifact);
    useEffect(() => {
        currentArtifact.current = artifact;
    }, [artifact]);
    const [pythonState, setPythonState] = useState<PythonState>({ status: 'idle' });
    useEffect(() => {
        if (!artifact || pythonState.artifact === artifact) return;
        setPythonState({ status: 'loading', artifact: artifact });
        void generatePythonSource(artifact).then(
            (source) => {
                if (currentArtifact.current === artifact) {
                    setPythonState({ status: 'ready', artifact: artifact, source });
                }
            },
            (error: unknown) => {
                if (currentArtifact.current === artifact) {
                    setPythonState({
                        status: 'error',
                        artifact: artifact,
                        message: error instanceof Error ? error.message : String(error),
                    });
                }
            },
        );
    }, [artifact, pythonState.status]);
    if (pythonState.status === 'ready' && pythonState.artifact === artifact) {
        return <SourceViewer language="python" source={pythonState.source} path="file:///playground.py" />;
    } else if (pythonState.status === 'error' && pythonState.artifact === artifact) {
        return (
            <div className={styles['compiled-placeholder']}>
                <div className={styles['compiled-error']}>{pythonState.message}</div>
                <button onClick={() => setPythonState({ status: 'idle' })}>重试</button>
            </div>
        );
    } else {
        return <div className={styles['compiled-placeholder']}>正在加载 Pyodide 并生成 Python 源代码…</div>;
    }
}
