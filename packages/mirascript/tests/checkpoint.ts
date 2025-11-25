import test from 'ava';
import { compile, configCheckpoint } from '@mirascript/mirascript';

test('configure timeout', (t) => {
    t.throws(() => configCheckpoint(0), {
        instanceOf: RangeError,
        message: 'Invalid timeout value',
    });
    t.throws(() => configCheckpoint(-100), {
        instanceOf: RangeError,
        message: 'Invalid timeout value',
    });
    t.throws(() => configCheckpoint(Number.NaN), {
        instanceOf: RangeError,
        message: 'Invalid timeout value',
    });
    t.notThrows(() => configCheckpoint(50));
    t.notThrows(() => configCheckpoint(1000));
    t.notThrows(() => configCheckpoint());
    t.notThrows(() => configCheckpoint(undefined));
});

test('callstack limit', async (t) => {
    const script = await compile(`
        fn c { c() }
        c()
    `);
    for (let i = 0; i < 1000; i++) {
        // 多次溢出异常
        t.throws(() => script(), {
            instanceOf: RangeError,
            message: 'Maximum call depth exceeded',
        });
    }

    // 内部状态未被破坏，正常递归
    const script2 = await compile(`
        fn f(n) {
            if n <= 0 {
                0
            } else {
                f(n - 1) + 1
            }
        }
        f(100)
    `);
    t.is(script2(), 100);
});
