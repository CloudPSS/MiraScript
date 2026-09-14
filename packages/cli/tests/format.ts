import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'ava';
import { run } from './_run.ts';

test('formats stdin with width options and reports syntax errors', async (t) => {
    const formatted = await run(['format', '--line-width', '12', '-'], 'let x=[1,2,3,4];');
    t.is(formatted.code, 0);
    t.regex(formatted.stdout, /let x = \[\n {2}1,/);

    const invalid = await run(['format', '-'], 'let x=[1,2,;');
    t.is(invalid.code, 1);
    t.regex(invalid.stderr, /格式化失败/);

    const empty = await run(['format', '-']);
    t.is(empty.code, 0);
    t.is(empty.stdout, '\n');

    const singleLine = await run(['format', '-'], '  \n  \n[1,2]\n  \n');
    t.is(singleLine.code, 0);
    t.is(singleLine.stdout, '[1, 2]');
});

test('check and write use deterministic exit codes', async (t) => {
    const directory = await mkdtemp(path.join(tmpdir(), 'mirascript-format-'));
    const file = path.join(directory, 'input.mira');
    try {
        await writeFile(file, 'let x=[1,2,3,4];', 'utf8');
        t.is((await run(['format', '--check', file])).code, 1);
        t.is((await run(['format', '--write', file])).code, 0);
        t.is((await run(['format', '--check', file])).code, 0);
        t.is(await readFile(file, 'utf8'), 'let x = [1, 2, 3, 4];');
    } finally {
        await rm(directory, { recursive: true, force: true });
    }
});

test('rejects invalid argument combinations and unmatched globs', async (t) => {
    t.is((await run(['format', '--write', '--check', 'missing.mira'])).code, 2);
    t.is((await run(['format', 'definitely-not-present-*.mira'])).code, 2);
    t.is((await run(['format', '--check', '-'])).code, 2);
});

test('infers template mode from the file extension and permits an explicit override', async (t) => {
    const directory = await mkdtemp(path.join(tmpdir(), 'mirascript-format-template-'));
    const file = path.join(directory, 'input.miratpl');
    try {
        await writeFile(file, 'Hello, $("World")!', 'utf8');
        const inferred = await run(['format', file]);
        t.is(inferred.code, 0);
        t.is(inferred.stdout, 'Hello, $("World")!');

        t.is((await run(['format', '--write', file])).code, 0);
        t.is(await readFile(file, 'utf8'), 'Hello, $("World")!');
        t.is((await run(['format', '--check', file])).code, 0);

        const overridden = await run(['format', '--no-template', file]);
        t.is(overridden.code, 1);
        t.regex(overridden.stderr, /格式化失败/);
    } finally {
        await rm(directory, { recursive: true, force: true });
    }
});
