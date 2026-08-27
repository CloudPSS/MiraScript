import test, { type ExecutionContext } from 'ava';
import {
    createVmContext,
    defineVmContextValue,
    getVmFunctionInfo,
    isVmExtern,
    isVmFunction,
    VmError,
    type VmFunction,
    type VmContext,
    type VmContextRecord,
    type VmValue,
} from '@mirascript/mirascript';
import { DefaultVmContext, lib } from '@mirascript/mirascript/subtle';

function checkContext(t: ExecutionContext, context: VmContext): Set<string> {
    const rk = `$$$RANDOM_GLOBAL_VALUE%${Math.random()}${Date.now()}`;

    t.true(context.has('to_string'));
    t.true(context.has('to_number'));
    t.false(context.has('to_boolean'));
    t.false(context.has(rk));

    const keys = new Set(context.keys());
    t.true(keys.has('to_string'));
    t.true(keys.has('to_number'));
    t.false(keys.has('to_boolean'));
    t.false(keys.has(rk));

    t.truthy(context.get('to_string'));
    t.truthy(context.get('to_number'));
    t.throws(() => context.get('to_boolean'), {
        message: `Global variable 'to_boolean' is not defined.`,
        instanceOf: VmError,
    });
    t.throws(() => context.get(rk), { message: `Global variable '${rk}' is not defined.`, instanceOf: VmError });

    t.is(context.describe('to_string'), lib.to_string.summary);
    t.is(context.describe('to_number'), lib.to_number.summary);
    t.is(context.describe('to_boolean'), undefined);
    t.is(context.describe(rk), undefined);

    defineVmContextValue(rk, rk, false, rk);
    const newKeys = new Set(context.keys());

    t.true(newKeys.has(rk));
    t.true(context.has(rk));
    t.is(context.get(rk), rk);
    t.is(context.describe(rk), rk);

    return newKeys;
}

test('DefaultContext', (t) => {
    t.true(Object.isFrozen(DefaultVmContext));
    checkContext(t, DefaultVmContext);
});

test('EmptyContext', (t) => {
    const context = createVmContext();
    t.is(context, DefaultVmContext);
    t.true(Object.isFrozen(context));
    checkContext(t, context);
});

const COMMON_ENV: VmContextRecord = {
    a: 1,
    b: [1, 2, 3],
    sin: 'sin',
    $ud: undefined,
    $nul: null,
    $set: new Set() as never,
    $date: new Date() as never,
    $fn: (() => 0) as never,
};

const COMMON_DESCRIBER = (key: string): string | undefined => key;

function checkValueContext(
    t: ExecutionContext,
    context: VmContext,
    keys: ReadonlySet<string>,
    env: VmContextRecord,
): void {
    t.is(context.get('sin'), 'sin');
    t.deepEqual(context.get('a'), 1);
    t.deepEqual(context.get('b'), [1, 2, 3]);

    t.true(context.has('a'));
    t.true(context.has('b'));
    t.false(context.has('c'));

    t.true(keys.has('a'));
    t.true(keys.has('b'));
    t.false(keys.has('c'));

    t.true(keys.has('$ud'));
    t.true(keys.has('$nul'));

    t.true(context.has('$ud'));
    t.true(context.has('$nul'));

    t.is(context.get('$ud'), null);
    t.is(context.get('$nul'), null);

    t.is(context.describe('sin'), 'sin');
    t.is(context.describe('a'), 'a');
    t.is(context.describe('b'), 'b');
    t.is(context.describe('c'), undefined);
    t.is(context.describe('$ud'), '$ud');
    t.is(context.describe('$nul'), '$nul');

    t.true(context.has('$set'));
    t.true(context.has('$date'));
    t.true(context.has('$fn'));
    t.is(context.describe('$set'), '$set');
    t.is(context.describe('$date'), '$date');
    t.is(context.describe('$fn'), '$fn');

    env['c'] = [1, 5];
    t.true(context.has('c'));
    t.is(context.get('c'), env['c']);
    t.is(context.describe('c'), 'c');

    delete env['c'];
    t.false(context.has('c'));
    t.throws(() => context.get('c'), { message: `Global variable 'c' is not defined.`, instanceOf: VmError });
    t.is(context.describe('c'), undefined);
}

test('ValueContext', (t) => {
    const env: VmContextRecord = { ...COMMON_ENV };
    const context = createVmContext(env, null, COMMON_DESCRIBER);

    const keys = checkContext(t, context);
    checkValueContext(t, context, keys, env);
});

test('Value2Context', (t) => {
    const env: VmContextRecord = { ...COMMON_ENV };
    const context = createVmContext(env, { e: [4, 5], $ud2: undefined, $nul2: null }, COMMON_DESCRIBER);

    const keys = checkContext(t, context);
    checkValueContext(t, context, keys, env);

    t.true(isVmExtern(context.get('e')));
    t.true(context.has('e'));
    t.true(keys.has('e'));

    t.true(keys.has('$ud2'));
    t.true(keys.has('$nul2'));

    t.is(context.get('$ud2'), null);
    t.is(context.get('$nul2'), null);

    t.is(context.describe('$ud2'), '$ud2');
    t.is(context.describe('$nul2'), '$nul2');
});

test('Value2Context extern only', (t) => {
    const extern: Record<string, unknown> = {
        $set: new Set(),
        $date: new Date(),
        $fn: () => 0,
    };
    const context = createVmContext(null, extern, COMMON_DESCRIBER);
    const keys = checkContext(t, context);

    t.true(context.has('$set'));
    t.true(context.has('$date'));
    t.true(context.has('$fn'));

    t.true(keys.has('$set'));
    t.true(keys.has('$date'));
    t.true(keys.has('$fn'));

    t.true(isVmExtern(context.get('$set')));
    t.false(isVmExtern(context.get('$date')));
    t.true(isVmExtern(context.get('$fn')));

    // Will be cached
    t.is(context.get('$set'), context.get('$set'));
    t.is(context.get('$date'), context.get('$date'));
    t.is(context.get('$fn'), context.get('$fn'));

    t.is(context.describe('$set'), '$set');
    t.is(context.describe('$date'), '$date');
    t.is(context.describe('$fn'), '$fn');
});

const FACTORY_ENV = (key: string): VmValue | undefined => {
    if (key.length === 1) return 'k_' + key;
    if (!key) return null;
    return undefined;
};

function checkFactoryContext(t: ExecutionContext, context: VmContext): void {
    t.is(context.get(''), null);
    t.throws(() => context.get('aa'), { message: `Global variable 'aa' is not defined.`, instanceOf: VmError });
    t.true(context.has(''));
    t.false(context.has('aa'));

    t.true(context.has('a'));
    t.true(context.has('b'));
    t.true(context.has('c'));
    t.true(context.has('d'));

    t.is(context.get('a'), 'k_a');
    t.is(context.get('b'), 'k_b');
    t.is(context.get('c'), 'k_c');
    t.is(context.get('d'), 'k_d');

    t.is(context.describe('a'), 'a');
    t.is(context.describe('b'), 'b');
    t.is(context.describe('c'), 'c');
    t.is(context.describe('d'), 'd');
    t.is(context.describe('aa'), undefined);
}

test('FactoryContext', (t) => {
    const context = createVmContext(FACTORY_ENV, () => ['d'], COMMON_DESCRIBER);
    const keys = checkContext(t, context);
    checkFactoryContext(t, context);

    t.false(keys.has('a'));
    t.false(keys.has('b'));
    t.false(keys.has('c'));
    t.true(keys.has('d'));
});

test('FactoryContextNoEnumerator', (t) => {
    const context = createVmContext(FACTORY_ENV, null, COMMON_DESCRIBER);
    const keys = checkContext(t, context);
    checkFactoryContext(t, context);

    t.false(keys.has('a'));
    t.false(keys.has('b'));
    t.false(keys.has('c'));
    t.false(keys.has('d'));
});

test('defineVmContextValue', (t) => {
    t.throws(() => {
        defineVmContextValue('to_string', 'value');
    });

    defineVmContextValue('$$FF', () => 0);
    const ff = DefaultVmContext.get('$$FF');
    t.true(isVmFunction(ff));
    t.is((ff as VmFunction)(), 0);
    t.is(getVmFunctionInfo(ff as VmFunction)?.fullName, `global.$$FF`);

    defineVmContextValue('$$FF', DefaultVmContext.get('to_string') as VmFunction, true);
    const ff2 = DefaultVmContext.get('$$FF');
    t.is(ff2, DefaultVmContext.get('to_string'));

    defineVmContextValue('$$FF', undefined as never, true);
    t.is(DefaultVmContext.get('$$FF'), null);
});
