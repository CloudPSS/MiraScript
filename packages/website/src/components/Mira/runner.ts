import { compile, createVmContext, VmFunction, type InputMode, type VmAny } from '@mirascript/mirascript';
import { lib, serializeForDisplay } from '@mirascript/mirascript/subtle';

/** 结果 */
export type Result = {
    type: 'result' | 'error' | 'log';
    content: Array<string | { value: string }>;
};

const printOptions = { ...lib.debug_print, prefix: [] };
/** 运行 MiraScript 代码 */
export async function runMiraScript(
    script: string,
    mode: InputMode,
    context: Record<string, VmAny> = {},
): Promise<Result[]> {
    const results: Result[] = [];
    try {
        const fn = await compile(script, {
            input_mode: mode,
            diagnostic_position_encoding: 'Utf32',
            fileName: 'live-code.' + (mode === 'Script' ? 'mira' : 'miratpl'),
            sourceMap: true,
            pretty: true,
        });
        const result = fn(
            createVmContext({
                ...context,
                debug_print: VmFunction((...args) => {
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
                                content.push({ value: serializeForDisplay(value) });
                                continue;
                            case '':
                            default:
                                content.push(typeof value == 'string' ? value : { value: serializeForDisplay(value) });
                                continue;
                        }
                    }
                    content.push(f.templates[f.values.length] ?? '');
                    results.push({ type: 'log', content });
                }, lib.debug_print),
            }),
        );
        if (result != null) {
            results.push({
                type: 'result',
                content: [{ value: serializeForDisplay(result) }],
            });
        }
    } catch (error) {
        results.push({
            type: 'error',
            content: [error instanceof Error ? error.message : String(error)],
        });
    }
    return results;
}
