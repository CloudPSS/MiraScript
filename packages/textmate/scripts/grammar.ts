import type { LanguageRegistration } from '@shikijs/types';
import {
    CONSTANT_KEYWORDS,
    CONTROL_KEYWORDS,
    KEYWORDS,
    NUMERIC_KEYWORDS,
    REG_IDENTIFIER,
    RESERVED_KEYWORDS,
} from '@mirascript/constants';

const SCOPE_SUFFIX = 'mira';
const IDENTIFIER = REG_IDENTIFIER.source;
const IDENTIFIER_END = String.raw`(?!\p{XID_Continue})`;
const MAX_VERBATIM_LENGTH = 16;
const BUILT_IN_TYPES = [
    'any',
    'unknown',
    'never',
    'boolean',
    'number',
    'string',
    'record',
    'array',
    'extern',
    'module',
    'nil',
];
const DOC_CONSTANT_IDENTIFIER = String.raw`(?:@+\p{XID_Continue}+|\p{Lu}[\p{XID_Continue}]*)`;

/**
 * Escape a literal value for insertion into an Oniguruma regular expression.
 */
function escapeRegex(value: string): string {
    return value.replaceAll(/[\\^$.*+?()[\]{}|]/g, String.raw`\$&`).replaceAll(/\s+/g, String.raw`\s+`);
}

/**
 * Build a longest-first regular-expression alternation.
 */
function alternatives(values: Iterable<string>): string {
    return [...values]
        .sort((a, b) => b.length - a.length)
        .map(escapeRegex)
        .join('|');
}

/**
 * Create a boundary-aware TextMate keyword rule.
 */
function keywordRule(values: readonly string[], name: string): { name: string; match: string } | null {
    if (values.length === 0) return null;
    return {
        name: `${name}.${SCOPE_SUFFIX}`,
        match: String.raw`(?<!\p{XID_Continue})(?:${alternatives(values)})${IDENTIFIER_END}`,
    };
}

const numericKeywordSet = new Set<string>(NUMERIC_KEYWORDS);
const constantKeywords = CONSTANT_KEYWORDS.filter((keyword) => !numericKeywordSet.has(keyword));
const languageVariables = ['_', 'global'];
const wordOperators = ['in', 'is', 'and', 'or', 'not'];
const declarationKeywords = ['fn', 'op', 'let', 'const', 'mut', 'where'];
const moduleKeywords = ['mod', 'pub', 'use'];
const classifiedKeywords = new Set([
    ...CONSTANT_KEYWORDS,
    ...CONTROL_KEYWORDS,
    ...languageVariables,
    ...wordOperators,
    ...declarationKeywords,
    ...moduleKeywords,
    ...RESERVED_KEYWORDS,
    'type',
]);
const otherKeywords = KEYWORDS.filter((keyword) => !classifiedKeywords.has(keyword));

/**
 * Create interpolation rules for one exact dollar-sign width.
 */
function interpolationPatterns(dollarCount: number) {
    const dollars = String.raw`\$`.repeat(dollarCount);
    return [
        {
            name: `meta.interpolation.simple.${SCOPE_SUFFIX}`,
            match: `(${dollars})(${IDENTIFIER})`,
            captures: {
                1: { name: `punctuation.definition.interpolation.begin.${SCOPE_SUFFIX}` },
                2: { name: `variable.other.${SCOPE_SUFFIX}` },
            },
        },
        {
            name: `meta.interpolation.block.${SCOPE_SUFFIX}`,
            begin: String.raw`(${dollars}\{)`,
            beginCaptures: { 1: { name: `punctuation.definition.interpolation.begin.${SCOPE_SUFFIX}` } },
            end: String.raw`\}`,
            endCaptures: { 0: { name: `punctuation.definition.interpolation.end.${SCOPE_SUFFIX}` } },
            patterns: [{ include: '#braced-content' }],
        },
        {
            name: `meta.interpolation.expression.${SCOPE_SUFFIX}`,
            begin: String.raw`(${dollars}\()`,
            beginCaptures: { 1: { name: `punctuation.definition.interpolation.begin.${SCOPE_SUFFIX}` } },
            end: String.raw`\)`,
            endCaptures: { 0: { name: `punctuation.definition.interpolation.end.${SCOPE_SUFFIX}` } },
            patterns: [
                {
                    begin: ':(?!:)',
                    beginCaptures: { 0: { name: `punctuation.separator.format.${SCOPE_SUFFIX}` } },
                    end: String.raw`(?=\))`,
                    contentName: `string.unquoted.format.${SCOPE_SUFFIX}`,
                    patterns: [{ include: '#format-content' }],
                },
                { include: '#interpolation-expression-content' },
            ],
        },
    ];
}

/**
 * Create a quoted-string rule with escapes and single-dollar interpolation.
 */
function normalString(quote: string, label: string) {
    const escapedQuote = escapeRegex(quote);
    return {
        name: `string.quoted.${label}.${SCOPE_SUFFIX}`,
        begin: escapedQuote,
        beginCaptures: { 0: { name: `punctuation.definition.string.begin.${SCOPE_SUFFIX}` } },
        end: escapedQuote,
        endCaptures: { 0: { name: `punctuation.definition.string.end.${SCOPE_SUFFIX}` } },
        patterns: [
            { name: `constant.character.escape.unicode.${SCOPE_SUFFIX}`, match: String.raw`\\u\{[0-9A-Fa-f]+\}` },
            { name: `constant.character.escape.hex.${SCOPE_SUFFIX}`, match: String.raw`\\x[0-9A-Fa-f]{2}` },
            { name: `constant.character.escape.${SCOPE_SUFFIX}`, match: '\\\\[\\\\\'"\\`$rntbfv0]' },
            { name: `invalid.illegal.escape.${SCOPE_SUFFIX}`, match: String.raw`\\.` },
            ...interpolationPatterns(1),
        ],
    };
}

/**
 * Create an exact-width MiraScript verbatim-string rule.
 */
function verbatimString(atCount: number, quote: string, label: string) {
    const ats = '@'.repeat(atCount);
    const escapedQuote = escapeRegex(quote);
    return {
        name: `string.quoted.${label}.verbatim.${SCOPE_SUFFIX}`,
        begin: `(?<!@)${ats}${escapedQuote}`,
        beginCaptures: { 0: { name: `punctuation.definition.string.begin.${SCOPE_SUFFIX}` } },
        end: `${escapedQuote}${ats}(?!@)`,
        endCaptures: { 0: { name: `punctuation.definition.string.end.${SCOPE_SUFFIX}` } },
        patterns: interpolationPatterns(atCount),
    };
}

/**
 * Build normal and 1-16 marker verbatim-string rules for every quote style.
 */
function stringPatterns() {
    const quotes = [
        ['"', 'double'],
        ["'", 'single'],
        ['`', 'backtick'],
    ];
    return [
        ...Array.from({ length: MAX_VERBATIM_LENGTH }, (_, index) => MAX_VERBATIM_LENGTH - index).flatMap((atCount) =>
            quotes.map(([quote, label]) => verbatimString(atCount, quote, label)),
        ),
        ...quotes.map(([quote, label]) => normalString(quote, label)),
    ];
}

/**
 * Build the shared repository used by source and template grammars.
 */
function sourceRepository(): LanguageRegistration['repository'] {
    const keywordPatterns = [
        keywordRule(NUMERIC_KEYWORDS, 'constant.numeric.language'),
        keywordRule(constantKeywords, 'constant.language'),
        keywordRule(CONTROL_KEYWORDS, 'keyword.control'),
        keywordRule(wordOperators, 'keyword.operator.wordlike'),
        keywordRule(declarationKeywords, 'keyword.declaration'),
        keywordRule(moduleKeywords, 'keyword.control.module'),
        keywordRule(RESERVED_KEYWORDS, 'keyword.reserved'),
        keywordRule(languageVariables, 'variable.language'),
        keywordRule(otherKeywords, 'keyword.other'),
    ].filter((p) => p != null);

    return {
        source: {
            patterns: [
                { include: '#comments' },
                { include: '#strings' },
                { include: '#function-declarations' },
                { include: '#module-declarations' },
                { include: '#for-bindings' },
                { include: '#record-properties' },
                { include: '#member-access' },
                { include: '#type-keyword' },
                { include: '#keywords' },
                { include: '#function-calls' },
                { include: '#numbers' },
                { include: '#identifiers' },
                { include: '#operators' },
                { include: '#punctuation' },
            ],
        },
        comments: {
            patterns: [
                { name: `comment.line.double-slash.${SCOPE_SUFFIX}`, match: '//.*$' },
                {
                    name: `comment.block.documentation.${SCOPE_SUFFIX}`,
                    begin: String.raw`/\*\*(?!/)`,
                    end: String.raw`\*/`,
                    patterns: [
                        {
                            name: `storage.type.class.documentation.${SCOPE_SUFFIX}`,
                            match: String.raw`@(param|returns)\b`,
                        },
                        { name: `markup.bold.documentation.${SCOPE_SUFFIX}`, match: String.raw`\*\*[^*]+\*\*` },
                        { name: `markup.italic.documentation.${SCOPE_SUFFIX}`, match: String.raw`\*[^*]+\*` },
                        { name: `constant.character.escape.documentation.${SCOPE_SUFFIX}`, match: String.raw`\\\*` },
                    ],
                },
                { name: `comment.block.${SCOPE_SUFFIX}`, begin: String.raw`/\*`, end: String.raw`\*/` },
            ],
        },
        'format-content': {
            patterns: [
                { name: `constant.character.escape.format.${SCOPE_SUFFIX}`, match: String.raw`\\.` },
                {
                    begin: String.raw`\[`,
                    end: String.raw`\]`,
                    name: `string.unquoted.format.character-class.${SCOPE_SUFFIX}`,
                    patterns: [{ name: `constant.character.escape.format.${SCOPE_SUFFIX}`, match: String.raw`\\.` }],
                },
                {
                    begin: String.raw`\(`,
                    end: String.raw`\)`,
                    name: `string.unquoted.format.group.${SCOPE_SUFFIX}`,
                    patterns: [{ include: '#format-content' }],
                },
            ],
        },
        strings: { patterns: stringPatterns() },
        'function-declarations': {
            patterns: [
                {
                    name: `meta.function.declaration.${SCOPE_SUFFIX}`,
                    begin: String.raw`(?<!\p{XID_Continue})(fn)(\s+)(${IDENTIFIER})(\s*)(\()`,
                    beginCaptures: {
                        1: { name: `keyword.declaration.function.${SCOPE_SUFFIX}` },
                        3: { name: `entity.name.function.${SCOPE_SUFFIX}` },
                        5: { name: `punctuation.section.parameters.begin.${SCOPE_SUFFIX}` },
                    },
                    end: String.raw`\)`,
                    endCaptures: { 0: { name: `punctuation.section.parameters.end.${SCOPE_SUFFIX}` } },
                    patterns: [{ include: '#parameters' }],
                },
                {
                    name: `meta.function.declaration.${SCOPE_SUFFIX}`,
                    match: String.raw`(?<!\p{XID_Continue})(fn)(\s+)(${IDENTIFIER})`,
                    captures: {
                        1: { name: `keyword.declaration.function.${SCOPE_SUFFIX}` },
                        3: { name: `entity.name.function.${SCOPE_SUFFIX}` },
                    },
                },
                {
                    name: `meta.function.expression.parameters.${SCOPE_SUFFIX}`,
                    begin: String.raw`(?<!\p{XID_Continue})(fn)(\s*)(\()`,
                    beginCaptures: {
                        1: { name: `keyword.declaration.function.${SCOPE_SUFFIX}` },
                        3: { name: `punctuation.section.parameters.begin.${SCOPE_SUFFIX}` },
                    },
                    end: String.raw`\)`,
                    endCaptures: { 0: { name: `punctuation.section.parameters.end.${SCOPE_SUFFIX}` } },
                    patterns: [{ include: '#parameters' }],
                },
            ],
        },
        parameters: {
            patterns: [
                {
                    match: String.raw`(?<!\p{XID_Continue})(mut)(\s+)(${IDENTIFIER})`,
                    captures: {
                        1: { name: `keyword.declaration.mutable.${SCOPE_SUFFIX}` },
                        3: { name: `variable.emphasis.${SCOPE_SUFFIX}` },
                    },
                },
                {
                    name: `keyword.declaration.mutable.${SCOPE_SUFFIX}`,
                    match: String.raw`(?<!\p{XID_Continue})mut${IDENTIFIER_END}`,
                },
                { name: `keyword.operator.spread.${SCOPE_SUFFIX}`, match: String.raw`\.\.` },
                { name: `variable.other.constant.emphasis.${SCOPE_SUFFIX}`, match: IDENTIFIER },
                { name: `punctuation.separator.parameter.${SCOPE_SUFFIX}`, match: ',' },
                {
                    begin: String.raw`\(`,
                    end: String.raw`\)`,
                    patterns: [{ include: '#parameters' }],
                },
                {
                    begin: String.raw`\[`,
                    end: String.raw`\]`,
                    patterns: [{ include: '#parameters' }],
                },
                { include: '#record-properties' },
            ],
        },
        'module-declarations': {
            patterns: [
                {
                    match: String.raw`(?<!\p{XID_Continue})(mod)(\s+)(${IDENTIFIER})`,
                    captures: {
                        1: { name: `keyword.control.module.${SCOPE_SUFFIX}` },
                        3: { name: `entity.name.namespace.${SCOPE_SUFFIX}` },
                    },
                },
            ],
        },
        'for-bindings': {
            patterns: [
                {
                    match: String.raw`(?<!\p{XID_Continue})(for)(\s+)(mut)(\s+)(${IDENTIFIER})(\s+)(in)${IDENTIFIER_END}`,
                    captures: {
                        1: { name: `keyword.control.loop.${SCOPE_SUFFIX}` },
                        3: { name: `keyword.declaration.mutable.${SCOPE_SUFFIX}` },
                        5: { name: `variable.other.readwrite.${SCOPE_SUFFIX}` },
                        7: { name: `keyword.control.loop.${SCOPE_SUFFIX}` },
                    },
                },
                {
                    match: String.raw`(?<!\p{XID_Continue})(for)(\s+)(${IDENTIFIER})(\s+)(in)${IDENTIFIER_END}`,
                    captures: {
                        1: { name: `keyword.control.loop.${SCOPE_SUFFIX}` },
                        3: { name: `variable.other.${SCOPE_SUFFIX}` },
                        5: { name: `keyword.control.loop.${SCOPE_SUFFIX}` },
                    },
                },
            ],
        },
        'record-properties': {
            patterns: [
                {
                    match: String.raw`(${IDENTIFIER})(\s*)(\??:)(?!:)`,
                    captures: {
                        1: { name: `variable.other.property.${SCOPE_SUFFIX}` },
                        3: { name: `punctuation.separator.key-value.${SCOPE_SUFFIX}` },
                    },
                },
            ],
        },
        'member-access': {
            patterns: [
                {
                    match: String.raw`(\.)(\s*)(\d+)`,
                    captures: {
                        1: { name: `punctuation.accessor.${SCOPE_SUFFIX}` },
                        3: { name: `variable.other.property.${SCOPE_SUFFIX}` },
                    },
                },
                {
                    match: `(\\.)(\\s*)(${IDENTIFIER})(\\s*)(!?)(?=\\s*(?:\\(|@*["'\`]))`,
                    captures: {
                        1: { name: `punctuation.accessor.${SCOPE_SUFFIX}` },
                        3: { name: `entity.name.function.member.${SCOPE_SUFFIX}` },
                        5: { name: `keyword.operator.non-null.${SCOPE_SUFFIX}` },
                    },
                },
                {
                    match: String.raw`(\.)(\s*)(${IDENTIFIER})`,
                    captures: {
                        1: { name: `punctuation.accessor.${SCOPE_SUFFIX}` },
                        3: { name: `variable.other.property.${SCOPE_SUFFIX}` },
                    },
                },
            ],
        },
        'function-calls': {
            patterns: [
                {
                    match: `(${IDENTIFIER})(\\s*)(!?)(?=\\s*(?:\\(|@*["'\`]))`,
                    captures: {
                        1: { name: `entity.name.function.${SCOPE_SUFFIX}` },
                        3: { name: `keyword.operator.non-null.${SCOPE_SUFFIX}` },
                    },
                },
            ],
        },
        'type-keyword': {
            patterns: [
                {
                    name: `keyword.operator.expression.${SCOPE_SUFFIX}`,
                    match: String.raw`(?<!\p{XID_Continue})type${IDENTIFIER_END}(?=\s*\(|\s+${IDENTIFIER})`,
                },
            ],
        },
        numbers: {
            patterns: [
                {
                    name: `constant.numeric.hex.${SCOPE_SUFFIX}`,
                    match: `0[xX][0-9A-Fa-f](?:[0-9A-Fa-f_]*[0-9A-Fa-f])?${IDENTIFIER_END}`,
                },
                {
                    name: `constant.numeric.octal.${SCOPE_SUFFIX}`,
                    match: `0[oO][0-7](?:[0-7_]*[0-7])?${IDENTIFIER_END}`,
                },
                {
                    name: `constant.numeric.binary.${SCOPE_SUFFIX}`,
                    match: `0[bB][01](?:[01_]*[01])?${IDENTIFIER_END}`,
                },
                {
                    name: `invalid.illegal.numeric.${SCOPE_SUFFIX}`,
                    match: String.raw`0[xXoObB]\p{XID_Continue}*`,
                },
                {
                    name: `constant.numeric.float.${SCOPE_SUFFIX}`,
                    match: String.raw`\d[\d_]*(?:\.[\d_]+)?(?:[eE][+-]?[\d_]*\d)?${IDENTIFIER_END}`,
                },
            ],
        },
        keywords: { patterns: keywordPatterns },
        identifiers: {
            patterns: [
                { name: `variable.other.constant.${SCOPE_SUFFIX}`, match: String.raw`@+\p{XID_Continue}+` },
                { name: `variable.other.${SCOPE_SUFFIX}`, match: IDENTIFIER },
            ],
        },
        operators: {
            patterns: [
                { name: `keyword.operator.spread.${SCOPE_SUFFIX}`, match: String.raw`\.\.<|\.\.` },
                { name: `keyword.operator.bind.${SCOPE_SUFFIX}`, match: '::' },
                { name: `keyword.operator.conditional.${SCOPE_SUFFIX}`, match: String.raw`\?:` },
                {
                    name: `keyword.operator.assignment.${SCOPE_SUFFIX}`,
                    match: String.raw`&&=|\|\|=|\?\?=|\+=|-=|\*=|/=|%=|\^=|=`,
                },
                {
                    name: `keyword.operator.comparison.${SCOPE_SUFFIX}`,
                    match: '===|!==|>=|<=|==|!=|=~|!~|>|<',
                },
                { name: `keyword.operator.logical.${SCOPE_SUFFIX}`, match: String.raw`&&|\|\||\?\?|!` },
                { name: `keyword.operator.arithmetic.${SCOPE_SUFFIX}`, match: String.raw`\+|-|\*|/|%|\^` },
            ],
        },
        punctuation: {
            patterns: [
                { name: `punctuation.section.braces.${SCOPE_SUFFIX}`, match: '[{}]' },
                { name: `punctuation.section.brackets.${SCOPE_SUFFIX}`, match: String.raw`[\[\]]` },
                { name: `punctuation.section.parens.${SCOPE_SUFFIX}`, match: '[()]' },
                { name: `punctuation.separator.${SCOPE_SUFFIX}`, match: '[,;:]' },
                { name: `punctuation.accessor.${SCOPE_SUFFIX}`, match: String.raw`\.` },
            ],
        },
        'braced-content': {
            patterns: [
                { begin: String.raw`\{`, end: String.raw`\}`, patterns: [{ include: '#braced-content' }] },
                { begin: String.raw`\(`, end: String.raw`\)`, patterns: [{ include: '#parenthesized-content' }] },
                { begin: String.raw`\[`, end: String.raw`\]`, patterns: [{ include: '#bracketed-content' }] },
                { include: '#source' },
            ],
        },
        'interpolation-expression-content': {
            patterns: [
                { begin: String.raw`\(`, end: String.raw`\)`, patterns: [{ include: '#parenthesized-content' }] },
                { begin: String.raw`\{`, end: String.raw`\}`, patterns: [{ include: '#braced-content' }] },
                { begin: String.raw`\[`, end: String.raw`\]`, patterns: [{ include: '#bracketed-content' }] },
                { include: '#comments' },
                { include: '#strings' },
                { include: '#function-declarations' },
                { include: '#module-declarations' },
                { include: '#for-bindings' },
                { include: '#member-access' },
                { include: '#type-keyword' },
                { include: '#keywords' },
                { include: '#function-calls' },
                { include: '#numbers' },
                { include: '#identifiers' },
                { include: '#operators' },
                { include: '#punctuation' },
            ],
        },
        'parenthesized-content': {
            patterns: [
                { begin: String.raw`\(`, end: String.raw`\)`, patterns: [{ include: '#parenthesized-content' }] },
                { begin: String.raw`\{`, end: String.raw`\}`, patterns: [{ include: '#braced-content' }] },
                { begin: String.raw`\[`, end: String.raw`\]`, patterns: [{ include: '#bracketed-content' }] },
                { include: '#source' },
            ],
        },
        'bracketed-content': {
            patterns: [
                { begin: String.raw`\[`, end: String.raw`\]`, patterns: [{ include: '#bracketed-content' }] },
                { begin: String.raw`\{`, end: String.raw`\}`, patterns: [{ include: '#braced-content' }] },
                { begin: String.raw`\(`, end: String.raw`\)`, patterns: [{ include: '#parenthesized-content' }] },
                { include: '#source' },
            ],
        },
    };
}

/**
 * Create the ordinary MiraScript TextMate grammar.
 */
export function createMiraScriptGrammar(): LanguageRegistration {
    return {
        name: 'mirascript',
        aliases: ['MiraScript', 'mira', 'Mira'],
        scopeName: 'source.mira',
        patterns: [{ include: '#source' }],
        repository: sourceRepository(),
    };
}

/**
 * Create the MiraScript template TextMate grammar.
 */
export function createMiraScriptTemplateGrammar(): LanguageRegistration {
    return {
        name: 'mirascript-template',
        aliases: ['MiraScript-Template', 'miratpl', 'MiraTpl'],
        scopeName: 'text.miratpl',
        patterns: [
            ...interpolationPatterns(1),
            { name: 'string.unquoted.template.mira', match: '[^$]+' },
            { name: 'string.unquoted.template.mira', match: String.raw`\$` },
        ],
        repository: sourceRepository(),
    };
}

/**
 * Create the generated-document and type-signature TextMate grammar.
 */
export function createMiraScriptDocGrammar(): LanguageRegistration {
    return {
        name: 'mirascript-doc',
        scopeName: 'source.mira.doc',
        patterns: [{ include: '#doc' }],
        repository: {
            doc: {
                patterns: [
                    { include: '#global-constants' },
                    { include: '#inline-parameter' },
                    { include: '#inline-label' },
                    { include: '#bindings' },
                    { include: '#function-signatures' },
                    { include: '#properties' },
                    { include: '#generic-types' },
                    { include: '#parameters' },
                    { include: '#return-type' },
                    { include: '#metadata' },
                    { include: 'source.mira' },
                ],
            },
            'global-constants': {
                patterns: [
                    {
                        match: String.raw`^(\x00)?(\(global\))(\s+)(${DOC_CONSTANT_IDENTIFIER})(?=\s*(?:=|;|$))`,
                        captures: {
                            2: { name: 'entity.name.label.mira' },
                            4: { name: 'variable.other.constant.mira' },
                        },
                    },
                    {
                        match: String.raw`^(\x00)?(${DOC_CONSTANT_IDENTIFIER})(?=\s*(?:=|;|$))`,
                        captures: { 2: { name: 'variable.other.constant.mira' } },
                    },
                ],
            },
            'inline-parameter': {
                patterns: [
                    {
                        match: String.raw`^(\x00)?(\(parameter(?:\s+pattern)?\))(\s+)(\.\.)?(mut)(\s+)(${IDENTIFIER})`,
                        captures: {
                            2: { name: 'entity.name.label.mira' },
                            4: { name: 'keyword.operator.spread.mira' },
                            5: { name: 'keyword.declaration.mutable.mira' },
                            7: { name: 'variable.emphasis.mira' },
                        },
                    },
                    {
                        match: String.raw`^(\x00)?(\(parameter(?:\s+pattern)?\))(\s+)(\.\.)?(\s*)(${IDENTIFIER})`,
                        captures: {
                            2: { name: 'entity.name.label.mira' },
                            4: { name: 'keyword.operator.spread.mira' },
                            6: { name: 'variable.other.constant.emphasis.mira' },
                        },
                    },
                ],
            },
            'inline-label': {
                patterns: [
                    {
                        match: String.raw`^(\x00)?(\([^():,\[\]<>|&\r\n]+\))(?=\s+\S)`,
                        captures: { 2: { name: 'entity.name.label.mira' } },
                    },
                ],
            },
            bindings: {
                patterns: [
                    {
                        match: String.raw`(?<!\p{XID_Continue})(let)(\s+)(mut)(\s+)(${IDENTIFIER})`,
                        captures: {
                            1: { name: 'keyword.declaration.variable.mira' },
                            3: { name: 'keyword.declaration.mutable.mira' },
                            5: { name: 'variable.other.readwrite.mira' },
                        },
                    },
                    {
                        match: String.raw`(?<!\p{XID_Continue})(let|const)(\s+)(${IDENTIFIER})`,
                        captures: {
                            1: { name: 'keyword.declaration.variable.mira' },
                            3: { name: 'variable.other.constant.mira' },
                        },
                    },
                ],
            },
            'function-signatures': {
                patterns: [
                    {
                        match: String.raw`(?<!\p{XID_Continue})(?:(pub)(\s+))?(fn)(\s+)(${IDENTIFIER})(?=\s*(?:<|\())`,
                        captures: {
                            1: { name: 'keyword.control.module.mira' },
                            3: { name: 'keyword.declaration.function.mira' },
                            5: { name: 'entity.name.function.mira' },
                        },
                    },
                ],
            },
            properties: {
                patterns: [
                    {
                        begin: String.raw`(${IDENTIFIER})(\s*)(\??:)`,
                        beginCaptures: {
                            1: { name: 'variable.other.property.mira' },
                            3: { name: 'punctuation.separator.type.mira' },
                        },
                        end: '$',
                        patterns: [{ include: '#metadata' }, { include: '#type-context' }],
                    },
                ],
            },
            metadata: {
                patterns: [
                    {
                        name: 'meta.documentation.type-hint.mira',
                        begin: String.raw`/\*\s*<`,
                        beginCaptures: { 0: { name: 'punctuation.definition.comment.begin.mira' } },
                        end: String.raw`>\s*\*/`,
                        endCaptures: { 0: { name: 'punctuation.definition.comment.end.mira' } },
                        patterns: [
                            { name: 'storage.modifier.extern.mira', match: String.raw`\bextern\b` },
                            { name: 'storage.type.function.mira', match: String.raw`\b(?:async\s+)?function\*?\b` },
                            { name: 'storage.type.class.mira', match: String.raw`\bclass\b` },
                            { name: 'entity.name.type.mira', match: IDENTIFIER },
                            { name: 'constant.numeric.mira', match: String.raw`\d+` },
                            { name: 'punctuation.separator.type.mira', match: '[().,<>]' },
                        ],
                    },
                ],
            },
            'reflection-type': {
                patterns: [
                    {
                        match: String.raw`(?<!\p{XID_Continue})(type)(\s*)(\()(\s*)(${IDENTIFIER})(\s*)(\))`,
                        captures: {
                            1: { name: 'keyword.operator.expression.mira' },
                            3: { name: 'punctuation.section.parens.begin.mira' },
                            5: { name: 'variable.other.mira' },
                            7: { name: 'punctuation.section.parens.end.mira' },
                        },
                    },
                ],
            },
            'type-context': {
                patterns: [
                    { include: '#reflection-type' },
                    { include: '#function-type' },
                    { include: '#generic-types' },
                    { include: '#type-grouping' },
                    { include: '#type-brackets' },
                    { include: '#type-atoms' },
                ],
            },
            'function-type': {
                patterns: [
                    {
                        begin: String.raw`(?<!\p{XID_Continue})(fn)${IDENTIFIER_END}(?=\s*(?:<|\())`,
                        beginCaptures: { 1: { name: 'storage.type.function.mira' } },
                        end: String.raw`(?=\s*(?:,|\)|$))`,
                        patterns: [
                            { include: '#generic-types' },
                            { include: '#parameters' },
                            { include: '#return-type' },
                            { include: '#type-context' },
                        ],
                    },
                ],
            },
            'generic-types': {
                patterns: [
                    {
                        begin: '<',
                        beginCaptures: { 0: { name: 'punctuation.definition.type-parameters.begin.mira' } },
                        end: '>',
                        endCaptures: { 0: { name: 'punctuation.definition.type-parameters.end.mira' } },
                        patterns: [
                            { include: '#reflection-type' },
                            { include: '#function-type' },
                            { include: '#generic-types' },
                            { include: '#type-grouping' },
                            { include: '#type-brackets' },
                            { include: '#type-atoms' },
                        ],
                    },
                ],
            },
            parameters: {
                patterns: [
                    {
                        begin: String.raw`\(`,
                        beginCaptures: { 0: { name: 'punctuation.section.parameters.begin.mira' } },
                        end: String.raw`\)`,
                        endCaptures: { 0: { name: 'punctuation.section.parameters.end.mira' } },
                        patterns: [
                            {
                                begin: String.raw`(\.\.)?(\s*)(mut)?(\s*)(${IDENTIFIER})(\s*)(\??:)(?=\s*fn${IDENTIFIER_END})`,
                                beginCaptures: {
                                    1: { name: 'keyword.operator.spread.mira' },
                                    3: { name: 'keyword.declaration.mutable.mira' },
                                    5: { name: 'entity.name.function.emphasis.mira' },
                                    7: { name: 'punctuation.separator.type.mira' },
                                },
                                end: String.raw`(?=,|\))`,
                                patterns: [{ include: '#type-context' }],
                            },
                            {
                                begin: String.raw`(\.\.)?(\s*)(mut)?(\s*)(${IDENTIFIER})(\s*)(\??:)`,
                                beginCaptures: {
                                    1: { name: 'keyword.operator.spread.mira' },
                                    3: { name: 'keyword.declaration.mutable.mira' },
                                    5: { name: 'variable.other.constant.emphasis.mira' },
                                    7: { name: 'punctuation.separator.type.mira' },
                                },
                                end: String.raw`(?=,|\))`,
                                patterns: [{ include: '#type-context' }],
                            },
                            {
                                match: String.raw`(\.\.)?(\s*)(mut)(\s+)(${IDENTIFIER})`,
                                captures: {
                                    1: { name: 'keyword.operator.spread.mira' },
                                    3: { name: 'keyword.declaration.mutable.mira' },
                                    5: { name: 'variable.emphasis.mira' },
                                },
                            },
                            { name: 'keyword.operator.spread.mira', match: String.raw`\.\.` },
                            { name: 'variable.other.constant.emphasis.mira', match: IDENTIFIER },
                            { name: 'punctuation.separator.parameter.mira', match: ',' },
                        ],
                    },
                ],
            },
            'type-grouping': {
                patterns: [
                    {
                        begin: String.raw`\(`,
                        beginCaptures: { 0: { name: 'punctuation.section.parens.begin.mira' } },
                        end: String.raw`\)`,
                        endCaptures: { 0: { name: 'punctuation.section.parens.end.mira' } },
                        patterns: [
                            {
                                begin: String.raw`(${IDENTIFIER})(\s*)(\??:)`,
                                beginCaptures: {
                                    1: { name: 'variable.other.property.mira' },
                                    3: { name: 'punctuation.separator.type.mira' },
                                },
                                end: String.raw`(?=,|\))`,
                                patterns: [{ include: '#type-context' }],
                            },
                            { include: '#type-context' },
                        ],
                    },
                ],
            },
            'type-brackets': {
                patterns: [
                    {
                        begin: String.raw`\[`,
                        beginCaptures: { 0: { name: 'punctuation.section.brackets.begin.mira' } },
                        end: String.raw`\]`,
                        endCaptures: { 0: { name: 'punctuation.section.brackets.end.mira' } },
                        patterns: [{ include: '#type-context' }],
                    },
                ],
            },
            'return-type': {
                patterns: [
                    {
                        begin: '->',
                        beginCaptures: { 0: { name: 'keyword.operator.type.mira' } },
                        end: String.raw`(?=,|\)|$)`,
                        patterns: [{ include: '#type-context' }],
                    },
                ],
            },
            'type-atoms': {
                patterns: [
                    { include: '#reflection-type' },
                    {
                        name: 'support.type.builtin.mira',
                        match: String.raw`(?<!\p{XID_Continue})(?:${alternatives(BUILT_IN_TYPES)})${IDENTIFIER_END}`,
                    },
                    {
                        name: 'constant.language.boolean.mira',
                        match: String.raw`(?<!\p{XID_Continue})(?:true|false)${IDENTIFIER_END}`,
                    },
                    { name: 'keyword.operator.type.mira', match: '[&|]' },
                    { name: 'keyword.operator.spread.mira', match: String.raw`\.\.` },
                    { name: 'entity.name.type.mira', match: IDENTIFIER },
                    { name: 'punctuation.section.brackets.mira', match: String.raw`[\[\]]` },
                    { name: 'punctuation.separator.type.mira', match: '[,.?:]' },
                    { include: 'source.mira#strings' },
                    { include: 'source.mira#numbers' },
                ],
            },
        },
    };
}

export const grammars: LanguageRegistration[] = [
    createMiraScriptGrammar(),
    createMiraScriptTemplateGrammar(),
    createMiraScriptDocGrammar(),
];
