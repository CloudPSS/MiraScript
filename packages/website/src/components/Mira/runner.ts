import {
    compile,
    createVmContext,
    VmFunction,
    type InputMode,
    type VmContext,
    type VmScript,
} from '@mirascript/mirascript';
import { lib, serializeForDisplay } from '@mirascript/mirascript/subtle';

let fileCounter = 1;
/** 创建名称 */
function createFileName(mode: InputMode, fileBaseName = 'live-code'): string {
    const counter = fileCounter++;
    const ext = mode === 'Script' ? 'mira' : 'miratpl';
    return `${fileBaseName}-${counter}.${ext}`;
}

/** 结果 */
export type Result = {
    type: 'result' | 'error' | 'log' | 'trace';
    timestamp: number;
    content: Array<string | { value: string; raw: unknown }>;
};

const printOptions = { ...lib.debug_print, prefix: [] };
let cache: { fileName: string; mode: InputMode; source: string; script: VmScript } | null = null;
/** 运行 MiraScript 代码 */
export async function runMiraScript(
    source: string,
    mode: InputMode,
    context?: VmContext,
    fileBaseName?: string,
    trace?: boolean,
): Promise<Result[]> {
    const results: Result[] = [];
    const start = performance.now();
    const now = () => performance.now() - start;
    const cacheHit = cache?.mode === mode && cache.source === source ? cache : null;
    const fileName = cacheHit?.fileName ?? createFileName(mode, fileBaseName);

    // 编译
    let script;
    try {
        const fn = await compile(source, {
            input_mode: mode,
            diagnostic_position_encoding: 'Utf32',
            fileName,
            sourceMap: true,
            pretty: true,
        });
        if (trace) {
            const t = now();
            results.push({
                type: 'trace',
                content: [`Compilation succeeded in ${t.toFixed(2)}ms.`],
                timestamp: t,
            });
        }
        if (cacheHit) {
            script = cacheHit.script;
        } else {
            script = fn;
            cache = { fileName, mode, source: source, script: fn };
        }
    } catch (error) {
        results.push({
            type: 'error',
            content: [error instanceof Error ? error.message : String(error)],
            timestamp: now(),
        });
    }
    if (!script) {
        return results;
    }

    // 运行
    try {
        const debug_print = VmFunction((...args) => {
            lib.debug_print(...args);
            const content: Result['content'] = [];
            const f = printOptions.parser(args);
            for (let i = 0; i < f.values.length; i++) {
                const template = f.templates[i] ?? '';
                content.push(template);
                const value = f.values[i]!;
                const format = f.formats[i] ?? '';
                switch (format) {
                    case '%s':
                    case '%d':
                    case '%f':
                        content.push(String(value));
                        continue;
                    case '%i':
                        content.push(Number(value).toFixed(0));
                        continue;
                    case '%o':
                    case '%O':
                        content.push({
                            value: serializeForDisplay(value),
                            raw: value,
                        });
                        continue;
                    case '':
                    default:
                        content.push(
                            typeof value == 'string' ? value : { value: serializeForDisplay(value), raw: value },
                        );
                        continue;
                }
            }
            content.push(f.templates[f.values.length] ?? '');
            results.push({ type: 'log', content, timestamp: now() });
        }, lib.debug_print);
        const s = now();
        const result = script(
            createVmContext((key) => {
                if (key === 'debug_print') return debug_print;
                return context?.get(key);
            }),
        );
        const t = now();
        if (trace) {
            results.push({
                type: 'trace',
                content: [`Execution succeeded in ${(t - s).toFixed(2)}ms.`],
                timestamp: t,
            });
        }
        if (result != null) {
            results.push({
                type: 'result',
                content: [{ value: serializeForDisplay(result), raw: result }],
                timestamp: t,
            });
        }
    } catch (error) {
        results.push({
            type: 'error',
            content: [error instanceof Error ? error.message : String(error)],
            timestamp: now(),
        });
    }
    return results;
}
