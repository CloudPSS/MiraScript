import test, { type ExecutionContext } from 'ava';
import {
    createVmContext,
    defineVmContextValue,
    getVmFunctionInfo,
    isVmExtern,
    isVmFunction,
    type VmFunction,
    type VmContext,
    VmError,
} from '@mirascript/mirascript';
import { DefaultVmContext } from '@mirascript/mirascript/subtle';

function checkContext(t: ExecutionContext, context: VmContext): Set<string> {
    const rk = `$$$RANDOM_GLOBAL_VALUE%${Math.random()}${Date.now()}`;

    t.true(context.has('to_string'));
    t.true(context.has('to_number'));
    t.true(context.has('to_boolean'));
    t.false(context.has(rk));

    const keys = new Set(context.keys());
    t.true(keys.has('to_string'));
    t.true(keys.has('to_number'));
    t.true(keys.has('to_boolean'));
    t.false(keys.has(rk));

    t.truthy(context.get('to_string'));
    t.truthy(context.get('to_number'));
    t.truthy(context.get('to_boolean'));
    t.throws(() => context.get(rk), { message: `Global variable '${rk}' is not defined.`, instanceOf: VmError });

    defineVmContextValue(rk, rk);
    const newKeys = new Set(context.keys());

    t.true(newKeys.has(rk));
    t.true(context.has(rk));
    t.is(context.get(rk), rk);

    return newKeys;
}

test('DefaultContext', (t) => {
    t.true(Object.isFrozen(DefaultVmContext));
    checkContext(t, DefaultVmContext);
});

test('EmptyContext', (t) => {
    const context = createVmContext();
    t.deepEqual(context, DefaultVmContext);
    t.not(context, DefaultVmContext);
    t.false(Object.isFrozen(context));
    checkContext(t, context);
});

test('ValueContext', (t) => {
    const context = createVmContext(
        {
            a: 1,
            b: [1, 2, 3],
            sin: 'sin',
            $ud: undefined,
            $nul: null,
            $set: new Set() as never,
            $date: new Date() as never,
            $fn: (() => 0) as never,
        },
        { c: [4, 5], $ud2: undefined, $nul2: null },
    );
    const keys = checkContext(t, context);

    t.is(context.get('sin'), 'sin');
    t.deepEqual(context.get('a'), 1);
    t.deepEqual(context.get('b'), [1, 2, 3]);
    t.true(isVmExtern(context.get('c')));

    t.true(context.has('a'));
    t.true(context.has('b'));
    t.true(context.has('c'));

    t.true(keys.has('a'));
    t.true(keys.has('b'));
    t.true(keys.has('c'));

    t.true(keys.has('$ud'));
    t.true(keys.has('$nul'));
    t.true(keys.has('$ud2'));
    t.true(keys.has('$nul2'));

    t.is(context.get('$ud'), null);
    t.is(context.get('$nul'), null);
    t.is(context.get('$ud2'), null);
    t.is(context.get('$nul2'), null);

    t.false(context.has('$set'));
    t.false(context.has('$date'));
    t.false(context.has('$fn'));
});

test('FactoryContext', (t) => {
    const context = createVmContext(
        (key) => (key.length === 1 ? 'k_' + key : !key ? null : undefined),
        () => ['d'],
    );

    t.is(context.get(''), null);
    t.throws(() => context.get('aa'), { message: `Global variable 'aa' is not defined.`, instanceOf: VmError });

    t.true(context.has(''));
    t.false(context.has('aa'));

    const keys = checkContext(t, context);

    t.true(context.has('a'));
    t.true(context.has('b'));
    t.true(context.has('c'));
    t.true(context.has('d'));

    t.false(keys.has('a'));
    t.false(keys.has('b'));
    t.false(keys.has('c'));
    t.true(keys.has('d'));

    t.is(context.get('a'), 'k_a');
    t.is(context.get('b'), 'k_b');
    t.is(context.get('c'), 'k_c');
    t.is(context.get('d'), 'k_d');
});

test('FactoryContextNoEnumerator', (t) => {
    const context = createVmContext((key) => (!key.startsWith('$') ? 'k_' + key : undefined));

    const keys = checkContext(t, context);

    t.true(context.has('a'));
    t.true(context.has('b'));
    t.true(context.has('c'));
    t.true(context.has('d'));

    t.false(keys.has('a'));
    t.false(keys.has('b'));
    t.false(keys.has('c'));
    t.false(keys.has('d'));

    t.is(context.get('a'), 'k_a');
    t.is(context.get('b'), 'k_b');
    t.is(context.get('c'), 'k_c');
    t.is(context.get('d'), 'k_d');
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
