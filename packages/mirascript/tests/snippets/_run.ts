import test from 'ava';
import { compile, type VmValue } from '@mirascript/mirascript';
import { createScript } from '@mirascript/mirascript/subtle';

type Options = {
    wrap?: boolean;
};

const compileAndRun = test.macro<[string, RegExp | VmValue, Options?]>({
    exec: async (t, code, expected, options) => {
        const expectError = expected instanceof RegExp ? { message: expected } : null;
        let scriptSource;
        {
            if (expectError) {
                await t.throwsAsync(async () => {
                    const script = await compile(code);
                    scriptSource = script.toString();
                    script();
                }, expectError);
            } else {
                const script = await compile(code);
                t.assert(script.length <= 1);
                scriptSource = script.toString();
                t.deepEqual(script(), expected);
            }
        }
        if (options?.wrap) {
            // wrap in a block to bypass fast route
            if (expectError) {
                await t.throwsAsync(async () => {
                    const script = await compile(`{\n${code}\n}`);
                    script();
                }, expectError);
            } else {
                const script = await compile(`{\n${code}\n}`);
                t.assert(script.length <= 1);
                t.deepEqual(script(), expected);
            }
        }
        if (scriptSource) {
            // create from source code
            const script = createScript(code, 'Script', scriptSource);
            if (expectError) {
                t.throws(() => script(), expectError);
            } else {
                t.deepEqual(script(), expected);
            }
        }
    },
    title: (providedTitle = 'test', code) => {
        if (code.length > 10) {
            return `${providedTitle} - ${code.slice(0, 10)}…`;
        }
        return `${providedTitle} - ${code}`;
    },
});

export const t = (title: string, code: string, expected: RegExp | VmValue, options?: Options): void => {
    test(title, compileAndRun, code, expected, options);
};

export const tw = (title: string, code: string, expected: RegExp | VmValue): void => {
    test(title, compileAndRun, code, expected, { wrap: true });
};
