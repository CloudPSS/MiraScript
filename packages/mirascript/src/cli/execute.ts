/* eslint-disable no-console */
import styles from 'ansi-styles';
import supportsColor from 'supports-color';
import { compile } from '../index.js';
import { createVmContext, VmFunction, type VmValue } from '../vm/index.js';
import { debug_print } from '../vm/lib/global/debug.js';
import { print } from './print.js';

/** 执行脚本 */
export async function execute(
    script: string,
    template: boolean,
    variables: Record<string, VmValue>,
    fileName: string,
): Promise<void> {
    try {
        const f = await compile(script, { input_mode: template ? 'Template' : 'Script', sourceMap: true, fileName });
        const r = f(
            createVmContext({
                debug_print: VmFunction((...values) => {
                    console.log(...debug_print.prefix, ...values.map((v) => (typeof v == 'string' ? v : print(v))));
                }, debug_print),
                ...variables,
            }),
        );
        if (template) {
            console.log(r);
        } else {
            console.log(print(r));
        }
    } catch (ex) {
        const { stack } = ex as Error;
        if (supportsColor.stderr) {
            console.error(styles.red.open + stack + styles.red.close);
        } else {
            console.error(stack);
        }
        process.exitCode = 2;
    }
}
