import test from 'ava';
import { tokenize, expectScope } from '../_engine.ts';

test('classifies declarations, parameters, properties, calls, and Unicode identifiers', (t) => {
    const tokens = tokenize('mod 数学 { fn 加法(mut 左, ..其余) { 对象.方法(左).属性 } }');
    expectScope(t, tokens, 'mod', 'keyword.module.mira');
    expectScope(t, tokens, '数学', 'entity.name.namespace.mira');
    expectScope(t, tokens, 'fn', 'keyword.declaration.function.mira');
    expectScope(t, tokens, '加法', 'entity.name.function.mira');
    expectScope(t, tokens, 'mut', 'keyword.declaration.mutable.mira');
    expectScope(t, tokens, '左', 'variable.emphasis.mira');
    expectScope(t, tokens, '其余', 'variable.other.constant.emphasis.mira');
    expectScope(t, tokens, '方法', 'entity.name.function.member.mira');
    expectScope(t, tokens, '属性', 'variable.other.property.mira');
});

test('classifies keyword and numeric families', (t) => {
    const tokens = tokenize('if true and value not in global { let @常量 = 0xCA_FE; 1.5e+2 }');
    expectScope(t, tokens, 'if', 'keyword.control.mira');
    expectScope(t, tokens, 'true', 'constant.language.mira');
    expectScope(t, tokens, 'and', 'keyword.operator.wordlike.mira');
    expectScope(t, tokens, 'not', 'keyword.operator.wordlike.mira');
    expectScope(t, tokens, 'in', 'keyword.operator.wordlike.mira');
    expectScope(t, tokens, 'global', 'variable.language.mira');
    expectScope(t, tokens, '@常量', 'variable.other.constant.mira');
    expectScope(t, tokens, '0xCA_FE', 'constant.numeric.hex.mira');
    expectScope(t, tokens, '1.5e+2', 'constant.numeric.float.mira');
});

test('does not classify control keywords followed by parentheses as functions', (t) => {
    const tokens = tokenize('if (condition) { case (value) { call(value) } }');
    expectScope(t, tokens, 'if', 'keyword.control.mira');
    expectScope(t, tokens, 'case', 'keyword.control.mira');
    expectScope(t, tokens, 'call', 'entity.name.function.mira');
    for (const keyword of ['if', 'case']) {
        const token = tokens.find((candidate) => candidate.text === keyword);
        t.false(token!.scopes.includes('entity.name.function.mira'));
    }
});

test('only classifies type as a keyword in its two contextual forms', (t) => {
    const tokens = tokenize('type(value); type Value; let type = 1; type + 1; object.type(); value::type();');
    expectScope(t, tokens, 'type', 'keyword.operator.expression.mira', 0);
    expectScope(t, tokens, 'type', 'keyword.operator.expression.mira', 1);
    expectScope(t, tokens, 'type', 'variable.other.mira', 2);
    expectScope(t, tokens, 'type', 'variable.other.mira', 3);
    expectScope(t, tokens, 'type', 'entity.name.function.member.mira', 4);
    expectScope(t, tokens, 'type', 'keyword.operator.expression.mira', 5);
});

test('marks invalid numeric and escape sequences', (t) => {
    const tokens = tokenize(String.raw`let a = 0xGG; let b = "\q";`);
    expectScope(t, tokens, '0xGG', 'invalid.illegal.numeric.mira');
    expectScope(t, tokens, String.raw`\q`, 'invalid.illegal.escape.mira');
});

test('keeps multiline rule state', (t) => {
    const tokens = tokenize('/* first\nsecond */\n"first ${\nvalue\n}"');
    expectScope(t, tokens, 'second ', 'comment.block.mira');
    expectScope(t, tokens, 'value', 'variable.other.mira');
});
