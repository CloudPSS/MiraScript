import test from 'ava';
import fs from 'node:fs/promises';
import path from 'node:path';

let mod1: typeof import('../dist/index.js') = {} as never;
let mod2: typeof import('../dist/index.js') = {} as never;

test.serial('model can be loaded multiple times', async (t) => {
    mod1 = await import('../dist/index.js');
    const distPath = path.resolve(import.meta.dirname, '../dist/');
    const distTempPath = path.resolve(import.meta.dirname, '../dist_temp/');
    await fs.cp(distPath, distTempPath, { recursive: true });
    t.teardown(() => fs.rm(distTempPath, { recursive: true, force: true }));
    mod2 = (await import('../dist_temp/index.js' as string)) as typeof import('../dist/index.js');
    t.not(mod1, mod2);
    t.not(mod1.compile, mod2.compile);
});

test('isVmScript works across instances', async (t) => {
    const s = `sin(PI / 2) + pow(2, 3)`;
    const s1 = await mod1.compile(s);
    const s2 = await mod2.compile(s);
    t.is(s1(), 9);
    t.is(s2(), 9);

    t.true(mod1.isVmScript(s1));
    t.true(mod1.isVmScript(s2));
});

test('isVmFunction works across instances', async (t) => {
    const sin1 = (await mod1.compile('sin'))();
    const sin2 = (await mod2.compile('sin'))();
    t.not(sin1, sin2);
    t.true(mod1.isVmFunction(sin1));
    t.true(mod1.isVmFunction(sin2));
});

test('isVmExtern and isVmModule work across instances', (t) => {
    const e1 = new mod1.VmExtern({});
    const e2 = new mod2.VmExtern({});
    t.true(e1 instanceof mod1.VmExtern);
    t.true(e2 instanceof mod2.VmExtern);
    t.false(e1 instanceof mod2.VmExtern);
    t.false(e2 instanceof mod1.VmExtern);
    t.true(mod1.isVmExtern(e1));
    t.true(mod1.isVmExtern(e2));

    const m1 = new mod1.VmModule('test1', {});
    const m2 = new mod2.VmModule('test2', {});
    t.true(m1 instanceof mod1.VmModule);
    t.true(m2 instanceof mod2.VmModule);
    t.false(m1 instanceof mod2.VmModule);
    t.false(m2 instanceof mod1.VmModule);
    t.true(mod1.isVmModule(m1));
    t.true(mod1.isVmModule(m2));
});
