import test from 'ava';
import { mirascriptLanguage } from '../../src/language.ts';
import { tokenize, expectScope } from '../_engine.ts';

test('separates documentation prefixes from bold and italic markup', (t) => {
    const tokens = tokenize(
        [
            '/**',
            ' * - *a*: 第一个操作数',
            ' * - **b**: 第二个操作数',
            ' * ```mirascript',
            ' * matrix.add([1, 2], [3, 4]) // [4, 6]',
            ' * ```',
            ' */',
        ].join('\n'),
    );
    expectScope(t, tokens, '*a*', 'markup.italic.documentation.mira');
    expectScope(t, tokens, '**b**', 'markup.bold.documentation.mira');
    for (const line of [1, 2, 3, 4, 5]) {
        const prefix = tokens.find((token) => token.line === line && token.text.trim() === '*');
        t.truthy(prefix, `Missing documentation prefix on line ${line}`);
        t.is(prefix!.scopes.at(-1), 'comment.block.documentation.mira');
    }
});

test('does not consume a documentation terminator as italic markup', (t) => {
    const tokens = tokenize('/**\n * *1*/\nlet x = 12;');
    expectScope(t, tokens, 'let', 'keyword.declaration.mira');
    expectScope(t, tokens, 'x', 'variable.other.mira');
    t.false(tokens.some((token) => token.text === '*1*' && token.scopes.includes('markup.italic.documentation.mira')));
});

test('ends a documentation fence before a prefixed comment terminator', (t) => {
    const tokens = tokenize('/**\n * ```mirascript\n * */\n * ```');
    const trailingLine = tokens.filter((token) => token.line === 3);
    t.true(trailingLine.length > 0);
    for (const token of trailingLine) {
        t.false(token.scopes.includes('comment.block.documentation.mira'), token.scopes.join(', '));
        t.false(token.scopes.includes('markup.fenced_code.block.mira'), token.scopes.join(', '));
    }
});

test('closes documentation before an embedded string can consume its terminator', (t) => {
    const tokens = tokenize("/**\n * ```mirascript\n * '*/\nlet recovered = 1;");
    expectScope(t, tokens, 'let', 'keyword.declaration.mira');
    expectScope(t, tokens, 'recovered', 'variable.other.mira');
    const recovered = tokens.find((token) => token.text === 'recovered');
    t.false(recovered!.scopes.includes('comment.block.documentation.mira'));
    t.false(recovered!.scopes.includes('markup.fenced_code.block.mira'));
});

test('preserves multiline embedded source state between safe documentation lines', (t) => {
    const tokens = tokenize("/**\n * ```mirascript\n * 'first\n * second'\n * ```\n */");
    expectScope(t, tokens, 'second', 'string.quoted.single.mira');
});

test('highlights documentation fences for the MiraScript name, aliases, and an omitted tag', (t) => {
    const tags = [undefined, mirascriptLanguage.name, ...(mirascriptLanguage.aliases ?? []), 'mIrAsCrIpT'];
    for (const tag of tags) {
        const tokens = tokenize(['/**', ` * \`\`\`${tag ?? ''}`, ' * matrix.identity(3)', ' * ```', ' */'].join('\n'));
        if (tag) expectScope(t, tokens, tag, 'fenced_code.block.language.mira');
        expectScope(t, tokens, 'identity', 'entity.name.function.member.mira');
    }
});

test('supports documentation fences with three or more backticks', (t) => {
    for (const [openingLength, closingLength] of [
        [3, 3],
        [4, 4],
        [8, 8],
        [12, 16],
    ] as const) {
        const opening = '`'.repeat(openingLength);
        const closing = '`'.repeat(closingLength);
        const tokens = tokenize(
            [
                '/**',
                ` * ${opening}mirascript`,
                ' * matrix.identity(3)',
                ` * ${closing}`,
                ' */',
                'let outside = 1;',
            ].join('\n'),
        );
        expectScope(t, tokens, 'identity', 'entity.name.function.member.mira');
        expectScope(t, tokens, 'outside', 'variable.other.mira');
    }
});

test('keeps unknown documentation fence tags as unparsed code blocks', (t) => {
    const tokens = tokenize(
        ['/**', ' * ````javascript', ' * let value = call(1);', ' * ````', ' */', 'let outside = 1;'].join('\n'),
    );
    expectScope(t, tokens, 'javascript', 'fenced_code.block.language.mira');
    const body = tokens.find((token) => token.line === 2 && token.text.includes('let value'));
    t.truthy(body);
    t.true(body!.scopes.includes('markup.fenced_code.block.mira'));
    t.true(body!.scopes.includes('markup.raw.block.mira'));
    t.false(body!.scopes.includes('keyword.declaration.mira'));
    t.false(body!.scopes.includes('entity.name.function.mira'));
    expectScope(t, tokens, 'outside', 'variable.other.mira');
});

test('highlights MiraScript fenced code inside documentation comments', (t) => {
    const tokens = tokenize(
        [
            '/**',
            ' * 创建一个单位矩阵',
            ' *',
            ' * - `..size`: 矩阵的维度',
            ' *',
            ' * ### 示例',
            ' * ```mirascript',
            ' * matrix.identity(3) // [[1, 0, 0], [0, 1, 0], [0, 0, 1]]',
            ' * ```',
            ' * @returns the matrix',
            ' */',
            'let outside = 2;',
        ].join('\n'),
    );
    expectScope(t, tokens, 'mirascript', 'fenced_code.block.language.mira');
    expectScope(t, tokens, 'identity', 'entity.name.function.member.mira');
    expectScope(t, tokens, '3', 'constant.numeric.float.mira');
    expectScope(t, tokens, '// [[1, 0, 0], [0, 1, 0], [0, 0, 1]]', 'comment.line.double-slash.mira');
    expectScope(t, tokens, '@returns', 'storage.type.class.documentation.mira');
    expectScope(t, tokens, 'let', 'keyword.declaration.mira');
    expectScope(t, tokens, 'outside', 'variable.other.mira');
});

test('ends an unterminated documentation fence at the comment boundary', (t) => {
    const tokens = tokenize('/**\n * ```mirascript\n * fn call() { nil }\n */\nlet recovered = 1;');
    expectScope(t, tokens, 'call', 'entity.name.function.mira');
    expectScope(t, tokens, 'let', 'keyword.declaration.mira');
    expectScope(t, tokens, 'recovered', 'variable.other.mira');
    const recovered = tokens.find((token) => token.text === 'recovered');
    t.false(recovered!.scopes.includes('comment.block.documentation.mira'));
});

test('highlights documentation comments inside documentation mode', (t) => {
    const tokens = tokenize(
        [
            '/**',
            ' * **bold** and *italic*',
            ' * ```mirascript',
            ' * matrix.identity(3)',
            ' * ```',
            ' */',
            'fn recovered(value: number) -> number',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, '**bold**', 'markup.bold.documentation.mira');
    expectScope(t, tokens, '*italic*', 'markup.italic.documentation.mira');
    expectScope(t, tokens, 'identity', 'entity.name.function.member.mira');
    expectScope(t, tokens, 'recovered', 'entity.name.function.mira');
    expectScope(t, tokens, 'number', 'support.type.builtin.mira');
});
