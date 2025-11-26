import supportsColor from 'supports-color';
import { VmError } from '../../../helpers/error.js';
import { toString } from '../../../helpers/convert/index.js';
import type { VmAny } from '../../types/index.js';
import { VmLib } from '../helpers.js';

/** 默认的序列化函数 */
function defaultSerializer(arg: VmAny, format: string): string | null {
    return null;
}

/** 序列化值 */
function serializeValue(arg: VmAny, format: string, serializer?: typeof defaultSerializer): string | null {
    if (serializer == null || serializer === defaultSerializer) {
        return defaultSerializer(arg, format);
    }
    try {
        return serializer(arg, format);
    } catch {
        return defaultSerializer(arg, format);
    }
}

export const debug_print = VmLib(
    (...args) => {
        if (args.length <= 1 || typeof args[0] != 'string' || !args[0].includes('%')) {
            // eslint-disable-next-line no-console
            console.log(...debug_print.prefix, ...args.map((v) => serializeValue(v, '', debug_print.serializer) ?? v));
            return;
        }
        const [prefix, ...additional] = debug_print.prefix;
        const [arg0, ...argsRest] = args;
        const format = `${prefix || ''} ${arg0}`;
        const values = [...additional, ...argsRest];
        const parts = format.split(/(%[%\w])/g);
        const messageToConsole: string[] = [];
        const valuesToConsole: unknown[] = [];
        let valIndex = 0;
        for (let i = 0; i < parts.length; i++) {
            if (i % 2 === 0) {
                // Regular string part
                messageToConsole.push(parts[i]!);
                continue;
            }
            // Specifier part
            const specifier = parts[i]!;
            if (specifier === '%%') {
                messageToConsole.push('%');
            } else {
                if (valIndex >= values.length) {
                    messageToConsole.push(specifier);
                    continue;
                }
                const arg = values[valIndex++]!;
                const f = serializeValue(arg, specifier, debug_print.serializer);
                if (f != null) {
                    messageToConsole.push('%s');
                    valuesToConsole.push(f);
                } else {
                    messageToConsole.push(specifier);
                    valuesToConsole.push(arg);
                }
            }
        }

        // Append any remaining arguments separated by spaces
        if (valIndex < values.length) {
            const remaining = values.slice(valIndex);
            valuesToConsole.push(...remaining.map((v) => serializeValue(v, '', debug_print.serializer) ?? v));
        }
        // eslint-disable-next-line no-console
        console.log(messageToConsole.join(''), ...valuesToConsole);
    },
    {
        summary: '打印调试信息到控制台',
        params: { '..args': '要打印的调试信息，可以是任意类型' },
        paramsType: { '..args': 'any[]' },
        returnsType: 'nil',
        examples: ['debug_print("value:", 42);'],
    },
    {
        prefix: ['MiraScript'] as readonly [prefix: string, ...additional: readonly string[]],
        serializer: defaultSerializer,
    },
);

export const panic = VmLib(
    (message: VmAny) => {
        // eslint-disable-next-line no-console
        if (message === undefined) console.error(...panic.prefix);
        // eslint-disable-next-line no-console
        else console.error(...panic.prefix, serializeValue(message, '', panic.serializer));
        const mgsStr = toString(message, null);
        const error = !mgsStr ? 'panic' : 'panic: ' + mgsStr;
        throw new VmError(error, undefined);
    },
    {
        summary: '产生错误，并打印错误信息到控制台',
        params: { message: '要打印的错误信息' },
        paramsType: { message: 'string' },
        returnsType: 'never',
        examples: ['panic("boom");'],
    },
    {
        prefix: ['MiraScript'] as readonly string[],
        serializer: defaultSerializer,
    },
);

if (typeof location != 'undefined') {
    const badge = '%cMiraScript%c';
    const common = 'display: inline-block; padding: 1px 4px; border-radius: 3px;';
    const reset = '';
    debug_print.prefix = [badge, `${common} background: #007acc; color: #fff;`, reset];
    panic.prefix = [badge, `${common} background: #d23d3d; color: #fff;`, reset];
} else {
    if (supportsColor.stdout) {
        debug_print.prefix = ['\u001B[44;37m MiraScript \u001B[0m'];
    }
    if (supportsColor.stderr) {
        panic.prefix = ['\u001B[41;37m MiraScript \u001B[0m'];
    }
}
