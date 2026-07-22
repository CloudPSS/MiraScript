import test from 'ava';
import type { ExecutionContext } from 'ava';
import fs from 'node:fs';
import {
    createVmContext,
    VmError,
    VmFunction,
    VmModule,
    compile,
    configCheckpoint,
    type VmScript,
} from '@mirascript/mirascript';
import { createScript } from '@mirascript/mirascript/subtle';

function createContext(t: ExecutionContext, timeout_fn: Array<[() => unknown, string]>, extern: boolean) {
    return createVmContext(
        {
            t_eq: VmFunction((a: unknown, b: unknown, message?: unknown) => {
                t.deepEqual(a, b, String(message || '') || undefined);
            }),
            t_ne: VmFunction((a: unknown, b: unknown, message?: unknown) => {
                t.notDeepEqual(a, b, String(message || '') || undefined);
            }),
            t_true: VmFunction((value: unknown, message?: unknown) => {
                t.true(value, String(message || '') || undefined);
            }),
            t_false: VmFunction((value: unknown, message?: unknown) => {
                t.false(value, String(message || '') || undefined);
            }),
            t_throws: VmFunction((fn: unknown, message?: unknown) => {
                t.throws(fn as () => unknown, { instanceOf: VmError }, String(message || '') || undefined);
            }),
            t_timeout: VmFunction((fn: unknown, message?: unknown) => {
                timeout_fn.push([fn as () => unknown, String(message || '') || 'Execution timed out']);
            }),
            t_never: VmFunction((message?: unknown) => {
                t.fail(String(message || '') || undefined);
            }),

            v_array: [],
            v_record: {},
        },
        {
            v_nil: null,
            v_true: true,
            v_false: false,
            v_number: 42,
            v_string: 'Hello, Mira!',
            v_fn: VmFunction(() => 'I am a function'),
            v_fn_another: VmFunction(() => 'I am another function'),
            v_module: new VmModule('v_module', {}),
            v_module_another: new VmModule('v_module_another', {}),
            has_extern: extern,
            ...(extern
                ? {
                      v_extern: {},
                      v_extern_another: {},
                  }
                : {}),
        },
    );
}

const TEST_DIR = new URL('../../../../tests', import.meta.url);

const runScript = (t: ExecutionContext, file: string, extern: boolean, script: VmScript) => {
    const timeout_fn: Array<[() => unknown, string]> = [];
    configCheckpoint(file.endsWith('_huge.mira') ? 5000 : 500);
    script(createContext(t, timeout_fn, extern));
    // 在脚本之后执行，否则脚本本身超时
    configCheckpoint();
    for (const [fn, message] of timeout_fn) {
        t.throws(fn, { instanceOf: RangeError, message: 'Execution timed out' }, message);
    }
};

const compileAndRun = test.macro<[string, boolean]>({
    exec: async (t, file, extern) => {
        const codeUrl = new URL(`./tests/${file}`, TEST_DIR);
        const code = await fs.promises.readFile(codeUrl, 'utf8');

        const script = await compile(code, { pretty: true, sourceMap: true, fileName: codeUrl.href });
        runScript(t, file, extern, script);
        const scriptCopy = createScript(code, 'Script', script.toString());
        runScript(t, file, extern, scriptCopy);
    },
    title: (providedTitle = 'test', code) => code || providedTitle,
});

export function run(extern: boolean): void {
    for (const file of fs.readdirSync(TEST_DIR, { encoding: 'utf8', recursive: true })) {
        const f = file.replaceAll('\\', '/');
        if (f.endsWith('.mira')) {
            const skip = f.includes('/_') || f.startsWith('_');
            if (skip) {
                test.skip(compileAndRun, f, extern);
            } else {
                test(compileAndRun, f, extern);
            }
        }
    }
}
