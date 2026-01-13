import test, { registerCompletionHandler } from 'ava';
import { compileWorker, destroyWorkerPool } from '../dist/compiler/worker-manager.js';

registerCompletionHandler(destroyWorkerPool);

test('compileWorker ok', async (t) => {
    const [script, diagnostics] = await compileWorker('let x = 42; x', {});
    t.true(script!.length > 0);
    t.true(diagnostics instanceof Uint32Array);
});

test('compileWorker error', async (t) => {
    const [script, diagnostics] = await compileWorker('}}{{', {});
    t.is(script, undefined);
    t.true(diagnostics instanceof Uint32Array);
    t.true(diagnostics.length > 0);
});
