import test from 'ava';
import { run } from './_run.ts';

test('shows help for language and standard library topics', async (t) => {
    const keyword = await run(['man', 'if']);
    t.is(keyword.code, 0);
    t.regex(keyword.stdout, /`if` 表达式/);
    t.is(keyword.stderr, '');

    const library = await run(['man', 'matrix.add']);
    t.is(library.code, 0);
    t.regex(library.stdout, /fn matrix\.add\(/);
    t.is(library.stderr, '');
});

test('rejects an unknown topic', async (t) => {
    const result = await run(['man', 'definitely-not-a-topic']);
    t.not(result.code, 0);
    t.regex(result.stderr, /Usage: mirascript man/);
});
