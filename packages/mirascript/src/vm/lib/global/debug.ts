import supportsColor from 'supports-color';
import { VmError } from '../../../helpers/error.js';
import { toString } from '../../../helpers/convert/to-string.js';
import type { VmAny } from '../../types/index.js';
import { VmLib } from '../helpers.js';

export const debug_print = VmLib(
    (...args) => {
        if (args.length > 1 && typeof args[0] == 'string' && args[0].includes('%')) {
            const [prefix, ...additional] = debug_print.prefix;
            const [format, ...argsRest] = args;
            const newPrefix = `${prefix} ${format}`;
            // eslint-disable-next-line no-console
            console.log(newPrefix, ...additional, ...argsRest);
        } else {
            // eslint-disable-next-line no-console
            console.log(...debug_print.prefix, ...args);
        }
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
    },
);

export const panic = VmLib(
    (message: VmAny) => {
        // eslint-disable-next-line no-console
        if (message === undefined) console.error(...panic.prefix);
        // eslint-disable-next-line no-console
        else console.error(...panic.prefix, message);
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
