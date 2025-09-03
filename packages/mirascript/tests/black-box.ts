import test from 'ava';
import fs from 'node:fs';
import { compile, createVmContext, VmError, VmFunction, VmModule } from '@mirascript/mirascript';

const TEST_DIR = new URL('../../../tests', import.meta.url);

const compileAndRun = test.macro<[string]>({
    exec: async (t, file) => {
        const codeUrl = new URL(`./tests/${file}`, TEST_DIR);
        const code = await fs.promises.readFile(codeUrl, 'utf8');

        const expectedUrl = new URL(`./tests/${file}.jsonl`, TEST_DIR);
        const expected = fs.existsSync(expectedUrl) ? await fs.promises.readFile(expectedUrl, 'utf8') : null;
        const script = await compile(code, { pretty: true, sourceMap: true, fileName: codeUrl.href });
        const timeout_fn: Array<() => unknown> = [];
        let result = '';
        script(
            createVmContext(
                {
                    t_eq: VmFunction((a: unknown, b: unknown) => {
                        t.deepEqual(a, b);
                    }),
                    t_ne: VmFunction((a: unknown, b: unknown) => {
                        t.notDeepEqual(a, b);
                    }),
                    t_true: VmFunction((value: unknown) => {
                        t.true(value);
                    }),
                    t_false: VmFunction((value: unknown) => {
                        t.false(value);
                    }),
                    t_throws: VmFunction((fn: unknown) => {
                        t.throws(fn as () => unknown, { instanceOf: VmError });
                    }),
                    t_timeout: VmFunction((fn: unknown) => {
                        timeout_fn.push(fn as () => unknown);
                    }),
                    t_snapshot: VmFunction((...values: unknown[]) => {
                        result += JSON.stringify(values) + '\n';
                    }),
                    t_never: VmFunction((message: unknown) => {
                        t.fail((message as string) || 'This should never be called');
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
        for (const fn of timeout_fn) {
            t.throws(fn, { instanceOf: RangeError, message: 'Execution timeout' });
        }
        if (expected != null) {
            t.is(result, expected, `Test ${file} output matches expected output`);
        } else if (result) {
            await fs.promises.writeFile(expectedUrl, result, 'utf8');
            t.pass(`Test ${file} output written to ${expectedUrl}`);
        }
    },
    title: (providedTitle = 'test', code) => code || providedTitle,
});

for (const file of fs.readdirSync(TEST_DIR, { encoding: 'utf8', recursive: true })) {
    if (file.endsWith('.mira')) {
        test(compileAndRun, file);
    }
}
