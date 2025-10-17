/* eslint-disable no-console */
import styles from 'ansi-styles';
import { compile } from '../index.ts';
import { createVmContext, VmFunction, type VmValue } from '../vm/index.ts';
import { debug_print } from '../vm/lib/global/debug.ts';
import { print } from './print.ts';

/** 执行脚本 */
export async function execute(script: string, template: boolean, variables: Record<string, VmValue>): Promise<void> {
    try {
        const f = await compile(script, { input_mode: template ? 'Template' : 'Script' });
        const r = f(
            createVmContext({
                debug_print: VmFunction((...values) => {
                    console.log(
                        '\u001B[46;30m MiraScript \u001B[0m',
                        ...values.map((v) => (typeof v == 'string' ? v : print(v))),
                    );
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
        console.error(styles.red.open + (ex as Error).message + styles.red.close);
        process.exitCode = 2;
    }
}
