import test from 'ava';
import { tokenize, expectScope } from '../_engine.ts';

test('handles nested interpolation and format strings', (t) => {
    const tokens = tokenize('"value ${ if ok { fn_call((1 + 2)) } } / $(value:>8[.]2f)"');
    expectScope(t, tokens, '${', 'punctuation.definition.template-expression.begin.mira');
    expectScope(t, tokens, 'fn_call', 'entity.name.function.mira');
    expectScope(t, tokens, '$(', 'punctuation.definition.template-expression.begin.mira');
    expectScope(t, tokens, ':', 'punctuation.separator.format.mira');
    expectScope(t, tokens, '>8', 'string.unquoted.format.mira');
});

test('scopes nested interpolation delimiters and strings', (t) => {
    const tokens = tokenize('"$(len([1,2,3])) ${let a = "x${\'y\'}"; a}"');
    expectScope(t, tokens, '(', 'punctuation.section.parens.begin.mira');
    expectScope(t, tokens, '[', 'punctuation.section.brackets.begin.mira');
    expectScope(t, tokens, ',', 'meta.embedded.expression.mira');
    expectScope(t, tokens, ']', 'punctuation.section.brackets.end.mira');
    expectScope(t, tokens, ')', 'punctuation.section.parens.end.mira');
    expectScope(t, tokens, '"', 'string.quoted.double.mira', 1);
    expectScope(t, tokens, "'", 'string.quoted.single.mira');
});

test('matches the exact interpolation width in verbatim strings', (t) => {
    for (const width of [1, 2, 3, 16]) {
        const ats = '@'.repeat(width);
        const dollars = '$'.repeat(width);
        const shorter = '$'.repeat(Math.max(1, width - 1));
        const tokens = tokenize(`${ats}"literal ${shorter}name ${dollars}name"${ats}`);
        expectScope(t, tokens, dollars, 'punctuation.definition.template-expression.begin.mira');
        expectScope(t, tokens, 'name', 'variable.other.mira', width === 1 ? 1 : 0);
        if (width > 1) {
            const literal = tokens.find((token) => token.text.includes(`${shorter}name`));
            t.true(literal!.scopes.includes('string.quoted.double.verbatim.mira'));
            t.false(literal!.scopes.includes('meta.interpolation.simple.mira'));
        }
    }
});

test('highlights template text and embedded MiraScript', (t) => {
    const tokens = tokenize('Hello $name: ${ fn_call(1) }', 'mirascript-template');
    expectScope(t, tokens, 'Hello ', 'string.unquoted.template.mira');
    expectScope(t, tokens, '$', 'punctuation.definition.template-expression.begin.mira');
    expectScope(t, tokens, 'name', 'variable.other.mira');
    expectScope(t, tokens, 'fn_call', 'entity.name.function.mira');
});
