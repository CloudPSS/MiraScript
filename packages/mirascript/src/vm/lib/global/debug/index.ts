import supportsColor from 'supports-color';
import { VmError } from '../../../../helpers/error.js';
import { toString } from '../../../../helpers/convert/index.js';
import type { VmAny } from '../../../types/index.js';
import { VmLib } from '../../helpers.js';
import { createPrintOptions, doPrint } from '../debug/utils.js';

export const debug_print = VmLib(
    (...args) => {
        doPrint(debug_print, args);
    },
    {
        summary: '打印调试信息到控制台',
        params: {
            '..args': { type: 'any[]', description: '要打印的值，可以是任意类型' },
        },
        returns: { type: 'nil' },
        examples: ['debug_print("value:", 42);'],
    },
    // eslint-disable-next-line no-console
    createPrintOptions(console.log.bind(console)),
);

export const panic = VmLib(
    (message: VmAny) => {
        doPrint(panic, message === undefined ? [] : [message]);
        const mgsStr = toString(message, null);
        const error = !mgsStr ? 'panic' : 'panic: ' + mgsStr;
        throw new VmError(error, undefined);
    },
    {
        summary: '产生错误，并打印错误信息到控制台',
        params: { message: { type: 'string', description: '要打印的错误信息' } },
        returns: { type: 'never' },
        examples: ['panic("boom");'],
    },
    // eslint-disable-next-line no-console
    createPrintOptions(console.error.bind(console)),
);

if (typeof location != 'undefined') {
    const badge = '%cMiraScript%c';
    const common = 'display: inline-block; padding: 1px 4px; border-radius: 3px;';
    const reset = '';
    debug_print.prefix = [badge, `${common} background: #007acc; color: #fff;`, reset];
    panic.prefix = [badge, `${common} background: #d23d3d; color: #fff;`, reset];
} else {
    if (supportsColor.stdout) {
        debug_print.prefix = ['\u{1B}[44;37m MiraScript \u{1B}[0m'];
    }
    if (supportsColor.stderr) {
        panic.prefix = ['\u{1B}[41;37m MiraScript \u{1B}[0m'];
    }
}
