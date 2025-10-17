import { VmError } from '../../error.ts';
import { $ToString } from '../../operations.ts';
import type { VmAny } from '../../types/index.ts';
import { VmLib } from '../_helpers.ts';

export const debug_print = VmLib(
    (...args) => {
        // eslint-disable-next-line no-console
        console.log('\u001B[46;30m MiraScript \u001B[0m', ...args);
    },
    {
        summary: '打印调试信息到控制台',
        params: { '..args': '要打印的调试信息，可以是任意类型' },
        paramsType: { '..args': 'any[]' },
        returnsType: 'nil',
        examples: ['debug_print("value:", 42);'],
    },
);

export const panic = VmLib(
    (message: VmAny) => {
        // eslint-disable-next-line no-console
        if (message === undefined) console.error('\u001B[41;37m MiraScript \u001B[0m');
        // eslint-disable-next-line no-console
        else console.error('\u001B[41;37m MiraScript \u001B[0m', message);
        const error = message == null ? 'panic' : 'panic: ' + $ToString(message);
        throw new VmError(error, undefined);
    },
    {
        summary: '产生错误，并打印错误信息到控制台',
        params: { message: '要打印的错误信息' },
        paramsType: { message: 'string' },
        returnsType: 'never',
        examples: ['panic("boom");'],
    },
);
