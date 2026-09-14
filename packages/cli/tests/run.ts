import test from 'ava';
import { run } from './_run.ts';

test('runs inline scripts and templates', async (t) => {
    const script = await run(['run', '--eval', '1 + 2']);
    t.is(script.code, 0);
    t.is(script.stdout, '3\n');
    t.is(script.stderr, '');

    const template = await run(['run', '--template', '--eval', 'Hello, $(name)!', '--variable', "name='Mira'"]);
    t.is(template.code, 0);
    t.is(template.stdout, 'Hello, Mira!\n');
    t.is(template.stderr, '');
});
