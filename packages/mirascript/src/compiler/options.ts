import type { TranspileOptions } from './types.js';

// 目前编译速度约 2000kB/s
export const WORKER_MIN_LEN = typeof Worker != 'function' ? Number.MAX_VALUE : 1024;

/** 补全代码生成选项。 */
export function normalizeTranspileOptions(options: TranspileOptions | undefined): TranspileOptions {
    options ??= {};
    if (options.sourceMap) {
        options.diagnostic_sourcemap = true;
        // https://tc39.es/ecma426/#sec-terms-and-definitions-colun
        options.diagnostic_position_encoding ??= 'Utf16';
    }
    options.input_mode ??= 'Script';
    return options;
}
