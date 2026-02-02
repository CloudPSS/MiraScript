import test from 'ava';
import { lib } from '@mirascript/mirascript/subtle';
import type { VmAny } from '../src/index.ts';
const { debug_print } = lib;

const s = test.macro<[Partial<typeof debug_print>, readonly VmAny[], readonly unknown[]]>({
    title: (providedTitle, options, args) => providedTitle || JSON.stringify(args),
    exec: async (t, options, args, expected) => {
        const opt = { ...debug_print, ...options };
        const formatResult = opt.parser(args);
        const formatted = await opt.formatter(formatResult);
        t.deepEqual(formatted, expected);
    },
});

{
    const t = (title: string, args: readonly VmAny[], expected: readonly unknown[]) => {
        test(`default ${title}`, s, {}, args, expected);
    };
    t('', [42, 'test', true], ['\u001B[44;37m MiraScript \u001B[0m %o %s %o', 42, 'test', true]);
    t('empty', [], ['\u001B[44;37m MiraScript \u001B[0m']);
    t('with string', ['test', 'value'], ['\u001B[44;37m MiraScript \u001B[0m test %s', 'value']);
    t('with format', ['%s%o', 3.14, { a: 1 }], ['\u001B[44;37m MiraScript \u001B[0m %s%o', 3.14, { a: 1 }]);
    t('with less args', ['Value: %d %s', 100], ['\u001B[44;37m MiraScript \u001B[0m Value: %d %%s', 100]);
    t('with more args', ['Value: %d', 1, 2, 3], ['\u001B[44;37m MiraScript \u001B[0m Value: %d %o %o', 1, 2, 3]);
    t('with %', ['Progress: 50% complete', 50], ['\u001B[44;37m MiraScript \u001B[0m Progress: 50%% complete %o', 50]);
    t(
        'with %%',
        ['Progress: 50%% complete', 50],
        ['\u001B[44;37m MiraScript \u001B[0m Progress: 50%% complete %o', 50],
    );
    t(
        'with %% and format',
        ['Progress: %d%% complete', 50],
        ['\u001B[44;37m MiraScript \u001B[0m Progress: %d%% complete', 50],
    );
    t('starts with format', ['%s is a string', 'This'], ['\u001B[44;37m MiraScript \u001B[0m %s is a string', 'This']);
    t('starts with %', ['% is a string', 'This'], ['\u001B[44;37m MiraScript \u001B[0m %% is a string %s', 'This']);
    t('starts with %%', ['%%%s is a string', 'This'], ['\u001B[44;37m MiraScript \u001B[0m %%%s is a string', 'This']);
}

{
    const t = (title: string, args: readonly VmAny[], expected: readonly unknown[]) => {
        test(`no-prefix ${title}`, s, { prefix: [] }, args, expected);
    };
    t('', [42, 'test', true], ['%o %s %o', 42, 'test', true]);
    t('empty', [], ['']);
    t('with string', ['test', 'value'], ['test %s', 'value']);
    t('with format', ['%s%o', 3.14, { a: 1 }], ['%s%o', 3.14, { a: 1 }]);
    t('with less args', ['Value: %d %s', 100], ['Value: %d %%s', 100]);
    t('with more args', ['Value: %d', 1, 2, 3], ['Value: %d %o %o', 1, 2, 3]);
    t('with %', ['Progress: 50% complete', 50], ['Progress: 50%% complete %o', 50]);
    t('with %%', ['Progress: 50%% complete', 50], ['Progress: 50%% complete %o', 50]);
    t('with %% and format', ['Progress: %d%% complete', 50], ['Progress: %d%% complete', 50]);
    t('starts with format', ['%s is a string', 'This'], ['%s is a string', 'This']);
    t('starts with %', ['% is a string', 'This'], ['%% is a string %s', 'This']);
    t('starts with %%', ['%%%s is a string', 'This'], ['%%%s is a string', 'This']);
}

{
    const t = (title: string, args: readonly VmAny[], expected: readonly unknown[]) => {
        test(`format-prefix ${title}`, s, { prefix: ['%s', 'FMT:'] }, args, expected);
    };
    t('', [42, 'test', true], ['%s %o %s %o', 'FMT:', 42, 'test', true]);
    t('empty', [], ['%s', 'FMT:']);
    t('with string', ['test', 'value'], ['%s test %s', 'FMT:', 'value']);
    t('with format', ['%s%o', 3.14, { a: 1 }], ['%s %s%o', 'FMT:', 3.14, { a: 1 }]);
    t('with less args', ['Value: %d %s', 100], ['%s Value: %d %%s', 'FMT:', 100]);
    t('with more args', ['Value: %d', 1, 2, 3], ['%s Value: %d %o %o', 'FMT:', 1, 2, 3]);
    t('with %', ['Progress: 50% complete', 50], ['%s Progress: 50%% complete %o', 'FMT:', 50]);
    t('with %%', ['Progress: 50%% complete', 50], ['%s Progress: 50%% complete %o', 'FMT:', 50]);
    t('with %% and format', ['Progress: %d%% complete', 50], ['%s Progress: %d%% complete', 'FMT:', 50]);
    t('starts with format', ['%s is a string', 'This'], ['%s %s is a string', 'FMT:', 'This']);
    t('starts with %', ['% is a string', 'This'], ['%s %% is a string %s', 'FMT:', 'This']);
    t('starts with %%', ['%%%s is a string', 'This'], ['%s %%%s is a string', 'FMT:', 'This']);
}

{
    const t = (title: string, args: readonly VmAny[], expected: readonly unknown[]) => {
        test(
            `async-serialize ${title}`,
            s,
            { prefix: [], serializer: (a) => Promise.resolve(String(a)) },
            args,
            expected,
        );
    };
    t('', [42, 'test', true], ['%s %s %s', String(42), 'test', String(true)]);
    t('empty', [], ['']);
    t('with string', ['test', 'value'], ['test %s', 'value']);
    t('with format', ['%s%s', 3.14, { a: 1 }], ['%s%s', String(3.14), String({ a: 1 })]);
    t('with less args', ['Value: %s %s', 100], ['Value: %s %%s', String(100)]);
    t('with more args', ['Value: %s', 1, 2, 3], ['Value: %s %s %s', String(1), String(2), String(3)]);
    t('with %', ['Progress: 50% complete', 50], ['Progress: 50%% complete %s', String(50)]);
    t('with %%', ['Progress: 50%% complete', 50], ['Progress: 50%% complete %s', String(50)]);
    t('with %% and format', ['Progress: %s%% complete', 50], ['Progress: %s%% complete', String(50)]);
    t('starts with format', ['%s is a string', 'This'], ['%s is a string', 'This']);
    t('starts with %', ['% is a string', 'This'], ['%% is a string %s', 'This']);
    t('starts with %%', ['%%%s is a string', 'This'], ['%%%s is a string', 'This']);
}
