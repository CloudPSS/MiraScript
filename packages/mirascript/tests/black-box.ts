import test from 'ava';
import fs from 'node:fs';
import { compile, createVmGlobal, VmError, type VmFunction } from 'mirascript';

const TEST_DIR = new URL('../../../tests', import.meta.url);

const compileAndRun = test.macro<[string]>({
    exec: async (t, file) => {
        const codeUrl = new URL(`./tests/${file}`, TEST_DIR);
        const code = await fs.promises.readFile(codeUrl, 'utf8');

        const expectedUrl = new URL(`./tests/${file}.jsonl`, TEST_DIR);
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
                        true: (value: unknown) => {
                            t.true(value);
                        },
                        false: (value: unknown) => {
                            t.false(value);
                        },
                        throws: (fn: VmFunction) => {
                            t.throws(fn, { instanceOf: VmError });
                        },
                        snapshot: (...values: unknown[]) => {
                            result += JSON.stringify(values) + '\n';
                        },
                        never: (message?: string) => {
                            t.fail(message || 'This should never be called');
                        },
                    },
                },
            ),
        );
        if (expected != null) {
            t.is(result, expected, `Test ${file} output matches expected output`);
        } else if (result) {
            await fs.promises.writeFile(expectedUrl, result, 'utf8');
            t.pass(`Test ${file} output written to ${expectedUrl}`);
        }
    },
    title: (providedTitle = 'test', code) => code || providedTitle,
});

for (const file of fs.readdirSync(TEST_DIR)) {
    if (file.endsWith('.mira')) {
        test(compileAndRun, file);
    }
}
