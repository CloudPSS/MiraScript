import { VmError } from '../../error.js';
import { $ToString } from '../../operations.js';
import type { VmAny } from '../../types/index.js';
import { VmLib } from '../_helpers.js';

export const debug_print = VmLib(
    (...args) => {
        // eslint-disable-next-line no-console
        console.log('\u001B[46;30m MiraScript \u001B[0m', ...args);
    },
    {
        summary: '打印调试信息到控制台',
        params: { '..args': '要打印的调试信息，可以是任意类型' },
        paramsType: { '..args': '[any]' },
        returnsType: 'nil',
    },
);

export const panic = VmLib(
    (message: VmAny) => {
        // eslint-disable-next-line no-console
        console.error('\u001B[41;37m MiraScript Panic \u001B[0m', message);
        throw new VmError($ToString(message), undefined);
    },
    {
        summary: '产生错误，并打印错误信息到控制台',
        params: { message: '要打印的错误信息' },
        paramsType: { message: 'string' },
        returnsType: 'never',
    },
);
