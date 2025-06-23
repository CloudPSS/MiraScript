import test from 'ava';
import fs from 'node:fs';
import { compile, createVmGlobal, VmError, type VmFunction } from 'mirascript';
import { fail } from 'node:assert';

const compileAndRun = test.macro<[string]>({
    exec: async (t, file) => {
        const codeUrl = new URL(`./black-box/${file}`, import.meta.url);
        const code = await fs.promises.readFile(codeUrl, 'utf8');

        const expectedUrl = new URL(`./black-box/${file}.jsonl`, import.meta.url);
        const expected = fs.existsSync(expectedUrl) ? await fs.promises.readFile(expectedUrl, 'utf8') : null;
        const script = await compile(code);
        let result = '';
        script(
            createVmGlobal(
                {},
                {
                    t: {
                        eq: (a: unknown, b: unknown) => {
                            t.deepEqual(a, b);
                        },
                        ne: (a: unknown, b: unknown) => {
                            t.notDeepEqual(a, b);
                        },
                        fail: (message?: string) => {
                            t.fail(message || 'Test failed');
                        },
                        throws: (fn: VmFunction) => {
                            t.throws(fn, { instanceOf: VmError });
                        },
                        snapshot: (...values: unknown[]) => {
                            result += JSON.stringify(values) + '\n';
                        },
                        true: (value: unknown) => {
                            t.true(value);
                        },
                        false: (value: unknown) => {
                            t.false(value);
                        },
                    },
                },
            ),
        );
        if (expected != null) {
            t.is(result, expected, `Test ${file} output matches expected output`);
        } else {
            await fs.promises.writeFile(expectedUrl, result, 'utf8');
            t.pass(`Test ${file} output written to ${expectedUrl}`);
        }
    },
    title: (providedTitle = 'test', code) => code || providedTitle,
});

for (const file of fs.readdirSync(new URL('./black-box', import.meta.url))) {
    if (file.endsWith('.mira')) {
        test(compileAndRun, file);
    }
}
