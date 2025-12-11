import test, { type ThrowsExpectation } from 'ava';
import { analyzeGlobalReferences, type GlobalReferenceChain } from '@mirascript/mirascript/subtle';

const analyze = test.macro<[string, GlobalReferenceChain[]]>({
    exec: (t, code, expected) => {
        t.deepEqual(analyzeGlobalReferences(code), expected);
    },
    title: (providedTitle = 'analyze', code) => `${providedTitle} ${code}`,
});

test(analyze, '  ', []);
test(analyze, '//', []);
test(analyze, 'if', []);
test(analyze, 'a + b + @c', [['a'], ['b'], ['@c']]);
test(analyze, 'a + ', []);
test(analyze, 'x100.0.1.2.01.0_1', []);
test(analyze, 'x100.1_1.2', []);
test(analyze, 'a.b.', []);
test(analyze, 'let a = 12; a + nil', []);
test(analyze, 'let a = 12; a + b', [['b']]);
test(analyze, 'a + b + a + a.b + b.c + a * a ! . b', [['a'], ['b'], ['a', 'b'], ['b', 'c']]);
test(analyze, '@@a + $b + c. $$ + $', [['@@a'], ['$b'], ['c', '$$'], ['$']]);
test(analyze, '变量.属性 + 变量2[索引] + 函数()', [['变量', '属性'], ['变量2'], ['索引'], ['函数']]);
test(analyze, 'a .b [1]+b', [['a', 'b'], ['b']]);
test(analyze, 'a. b["x"]+b', [['a', 'b'], ['b']]);
test(analyze, 'a.b[x] + b + c.12.3', [['a', 'b'], ['x'], ['b'], ['c', 12, 3]]);
test(analyze, 'sin(x) + y::z!. t!()::cos!() + w.123', [['sin'], ['x'], ['cos'], ['z', 't'], ['y'], ['w', 123]]);
