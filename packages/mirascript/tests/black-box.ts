import test from 'ava';
import fs from 'node:fs';
import { compile, createVmContext, VmError, VmFunction, VmModule, configCheckpoint } from '@mirascript/mirascript';

const TEST_DIR = new URL('../../../tests', import.meta.url);

const compileAndRun = test.macro<[string]>({
    exec: async (t, file) => {
        const codeUrl = new URL(`./tests/${file}`, TEST_DIR);
        const code = await fs.promises.readFile(codeUrl, 'utf8');

        const script = await compile(code, { pretty: true, sourceMap: true, fileName: codeUrl.href });
        const timeout_fn: Array<[() => unknown, string | undefined]> = [];
        configCheckpoint(file.endsWith('_huge.mira') ? 1000 : undefined);
        script(
            createVmContext(
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
                        timeout_fn.push([fn as () => unknown, String(message || '') || 'Execution timeout']);
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
                    has_extern: true,
                    v_extern: {},
                    v_extern_another: {},
                    has_module: true,
                    v_module: new VmModule('v_module', {}),
                    v_module_another: new VmModule('v_module_another', {}),
                },
            ),
        );
        // 在脚本之后执行，否则脚本本身超时
        for (const [fn, message] of timeout_fn) {
            t.throws(fn, { instanceOf: RangeError, message: 'Execution timeout' }, message);
        }
    },
    title: (providedTitle = 'test', code) => code || providedTitle,
});

for (const file of fs.readdirSync(TEST_DIR, { encoding: 'utf8', recursive: true })) {
    if (file.endsWith('.mira')) {
        test(compileAndRun, file);
    }
}
