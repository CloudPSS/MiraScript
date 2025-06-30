import type { VmScript } from '../vm/index.js';
import type { TranspileOptions, ScriptInput } from './types.js';
import { compile as compileCore } from './compile.js';
import { createScript } from './create-script.js';
import { compileFast } from './compile-fast.js';
import { DiagnosticCode, getDiagnosticMessage, parseDiagnostics, type SourceDiagnostic } from './utils.js';

export type { TranspileOptions, ScriptInput, InputMode } from './types.js';

// 目前编译速度约 2000kB/s
const WORKER_MIN_LEN = typeof process == 'object' && process.versions?.node != null ? Number.MAX_VALUE : 1024;

let worker: Promise<Worker> | undefined;
/** 获取 worker */
async function getWorker(): Promise<Worker> {
    if (worker) return worker;
    worker = new Promise((resolve, reject) => {
        const w = new Worker(new URL('#compiler/worker', import.meta.url), {
            type: 'module',
            name: '@mirascript/compiler',
        });
        w.addEventListener('error', reject);
        w.addEventListener('message', (msg) => {
            if (msg.data === 'ready') {
                resolve(w);
            }
        });
    });
    return worker;
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
async function compileWorker(...args: Parameters<typeof compileCore>): Promise<[string | undefined, Uint32Array]> {
    const worker = await getWorker();
    const seq = Math.random();
    worker.postMessage([seq, ...args]);
    return await new Promise<[string | undefined, Uint32Array]>((resolve, reject) => {
        const callback = (ev: MessageEvent) => {
            const data = ev.data as [number, string | undefined, Uint32Array] | [number, string];
            if (!Array.isArray(data)) return;
            const [retSeq, ...rest] = data;
            if (seq !== retSeq) return; // Ignore messages not matching the request
            worker.removeEventListener('message', callback);
            if (rest.length === 2) {
                resolve(rest);
            } else {
                reject(new Error(rest[0]));
            }
        };
        worker.addEventListener('message', callback);
    });
}
/** 生成诊断 range 的字符串 */
function formatRange(range: {
    startLineNumber: number;
    startColumn: number;
    endLineNumber: number;
    endColumn: number;
}): string {
    if (range.startLineNumber === range.endLineNumber) {
        if (range.startColumn === range.endColumn) {
            return `${range.startLineNumber}:${range.startColumn}`;
        }
        return `${range.startLineNumber}:${range.startColumn}-${range.endColumn}`;
    }
    return `${range.startLineNumber}:${range.startColumn}-${range.endLineNumber}:${range.endColumn}`;
}

/** 生成诊断消息的字符串 */
function formatDiagnostic(diagnostic: SourceDiagnostic): string {
    const range = formatRange(diagnostic.range);
    const codeName = DiagnosticCode[diagnostic.code] || `Unknown(${diagnostic.code})`;
    let message = getDiagnosticMessage(diagnostic.code);
    for (const ref of diagnostic.references) {
        const refRange = formatRange(ref.range);
        message += `\n    (${refRange}): ${getDiagnosticMessage(ref.code)}`;
    }
    return `  ${codeName}(${range}): ${message}`;
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
async function compileImpl(...args: Parameters<typeof compileCore>): Promise<VmScript> {
    const source = args[0];
    const [code, diagnostics] =
        source.length < WORKER_MIN_LEN ? await compileCore(...args) : await compileWorker(...args);
    if (!code) {
        const parsed = parseDiagnostics(
            ArrayBuffer.isView(source) ? new TextDecoder().decode(source) : source,
            diagnostics,
        );
        const messages = parsed.errors.map(formatDiagnostic);
        throw new Error(`Failed to compile:\n${messages.join('\n')}`);
    }
    return createScript(args[0], code);
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(source: ScriptInput, options: TranspileOptions = {}): Promise<VmScript> {
    if (typeof source == 'string') {
        const result = compileFast(source, options);
        if (result) {
            return result;
        }
    }
    return compileImpl(source, options);
}
