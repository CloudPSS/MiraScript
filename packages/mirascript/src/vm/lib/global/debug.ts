import { VmLib } from '../helpers.js';

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
