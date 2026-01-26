/* eslint-disable no-console */
import styles from 'ansi-styles';
import supportsColor from 'supports-color';
import { compile, createVmContext, type VmValue } from '@mirascript/mirascript';
import { lib } from '@mirascript/mirascript/subtle';
import { print } from './print.js';

const { panic, debug_print } = lib;

panic.serializer = debug_print.serializer = (arg, format) => {
    if (format === '%o' || format === '%O' || !format) {
        return print(arg);
    }
    return null;
};

/** 执行脚本 */
export async function execute(
    script: string,
    template: boolean,
    variables: Record<string, VmValue>,
    fileName: string,
): Promise<void> {
    try {
        const f = await compile(script, { input_mode: template ? 'Template' : 'Script', sourceMap: true, fileName });
        const r = f(createVmContext(variables));
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
