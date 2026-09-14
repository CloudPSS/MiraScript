import test from 'ava';
import { run } from './_run.ts';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { writeFile } from 'node:fs/promises';

test('runs inline scripts and templates', async (t) => {
    const script = await run(['run', '--eval', '1 + 2']);
    t.is(script.code, 0);
    t.snapshot(script.stdout);
    t.is(script.stderr, '');

    const template = await run(['run', '--template', '--eval', 'Hello, $(name)!', '--variable', "name='Mira'"]);
    t.is(template.code, 0);
    t.is(template.stdout, 'Hello, Mira!\n');
    t.is(template.stderr, '');
});

test('runs scripts from stdin', async (t) => {
    const script = await run(['run', '-'], '1 + 2');
    t.is(script.code, 0);
    t.snapshot(script.stdout);
    t.is(script.stderr, '');
});
test('runs script file', async (t) => {
    const p = path.join(tmpdir(), 'script.mira');
    await writeFile(p, '1 + 2');
    const script = await run(['run', p]);
    t.is(script.code, 0);
    t.snapshot(script.stdout);
    t.is(script.stderr, '');
});
