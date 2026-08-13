import type { LanguageRegistration } from '@shikijs/types';
import {
    CONSTANT_KEYWORDS,
    CONTROL_KEYWORDS,
    KEYWORDS,
    NUMERIC_KEYWORDS,
    REG_IDENTIFIER,
    RESERVED_KEYWORDS,
} from '@mirascript/constants';
import { mirascriptDocLanguage, mirascriptLanguage, mirascriptTemplateLanguage } from './language.ts';

const IDENTIFIER = REG_IDENTIFIER.source;
const IDENTIFIER_START = String.raw`(?<!\p{XID_Continue})`;
const IDENTIFIER_END = String.raw`(?!\p{XID_Continue})`;
const MAX_VERBATIM_LENGTH = 16;
const BUILT_IN_TYPES = [
    'any',
    'unknown',
    'never',

    'boolean',
    'true',
    'false',

    'number',
    'string',

    'record',
    'array',
    'extern',
    'module',

    'nil',
];
const DOC_CONSTANT_IDENTIFIER = String.raw`(?:@+\p{XID_Continue}+|\p{Lu}[\p{XID_Continue}]*)`;
const DOC_TAG_NAME = String.raw`[^>\r\n]*[^>\s]`;
const DOC_TAG_CONTENT = String.raw`(?:module|function|extern)(?:[ \t]+${DOC_TAG_NAME})?`;
const DOC_FIELD_NAME = String.raw`(?:${IDENTIFIER}|\d+|"(?:\\.|[^"\\\r\n])*"|'(?:\\.|[^'\\\r\n])*')`;
const DOC_TAG_NUMBER = String.raw`\d[\d_]*(?:\.[\d_]+)?(?:[eE][+-]?[\d_]*\d)?`;

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
const mirascriptFenceTags = alternatives([mirascriptLanguage.name, ...mirascriptLanguage.aliases]);
const documentationMiraFenceBegin = String.raw`^(\s*\*\s?)(\x60{3,})\s*((?i:${mirascriptFenceTags}))?\s*$`;
const documentationOtherFenceBegin = String.raw`^(\s*\*\s?)(\x60{3,})\s*(\S+)(?:\s+.*)?\s*$`;
const documentationFenceEnd = String.raw`^(?:(\s*\*\s?)(\2\x60*)\s*$)|(?=\*/)`;
const documentationMiraLine = String.raw`^(?!\s*\*\s*\x60{3,}\s*$)(?!.*\*/)(\s*\*\s?)`;
const documentationInjectionSelector =
    'L:comment.block.documentation.mira -markup.fenced_code.block.mira -meta.documentation.inline.mira';

/**
 * Create interpolation rules for one exact dollar-sign width.
 */
function interpolationPatterns(dollarCount: number) {
    const dollars = String.raw`\$`.repeat(dollarCount);
    return [
        {
            name: `meta.embedded.interpolation.simple.mira`,
            match: `(${dollars})(${IDENTIFIER})`,
            captures: {
                1: { name: `punctuation.definition.template-expression.begin.mira` },
                2: { name: `variable.other.mira` },
            },
        },
        {
            name: `meta.embedded.interpolation.block.mira`,
            begin: String.raw`(${dollars}\{)`,
            beginCaptures: { 1: { name: `punctuation.definition.template-expression.begin.mira` } },
            end: String.raw`\}`,
            endCaptures: { 0: { name: `punctuation.definition.template-expression.end.mira` } },
            patterns: [{ include: '#braced-content' }],
        },
        {
            name: `meta.embedded.interpolation.expression.mira`,
            begin: String.raw`(${dollars}\()`,
            beginCaptures: { 1: { name: `punctuation.definition.template-expression.begin.mira` } },
            end: String.raw`\)`,
            endCaptures: { 0: { name: `punctuation.definition.template-expression.end.mira` } },
            patterns: [
                {
                    begin: ':(?!:)',
                    beginCaptures: { 0: { name: `punctuation.separator.format.mira` } },
                    end: String.raw`(?=\))`,
                    contentName: `string.unquoted.format.mira`,
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
        name: `string.quoted.${label}.mira`,
        begin: escapedQuote,
        beginCaptures: { 0: { name: `punctuation.definition.string.begin.mira` } },
        end: escapedQuote,
        endCaptures: { 0: { name: `punctuation.definition.string.end.mira` } },
        patterns: [
            { name: `constant.character.escape.unicode.mira`, match: String.raw`\\u\{[0-9A-Fa-f]+\}` },
            { name: `constant.character.escape.hex.mira`, match: String.raw`\\x[0-9A-Fa-f]{2}` },
            { name: `constant.character.escape.mira`, match: '\\\\[\\\\\'"\\`$rntbfv0]' },
            { name: `invalid.illegal.escape.mira`, match: String.raw`\\.` },
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
        name: `string.quoted.${label}.verbatim.mira`,
        begin: `(?<!@)${ats}${escapedQuote}`,
        beginCaptures: { 0: { name: `punctuation.definition.string.begin.mira` } },
        end: `${escapedQuote}${ats}(?!@)`,
        endCaptures: { 0: { name: `punctuation.definition.string.end.mira` } },
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
    ] as const;
    return [
        ...Array.from({ length: MAX_VERBATIM_LENGTH }, (_, index) => MAX_VERBATIM_LENGTH - index).flatMap((atCount) =>
            quotes.map(([quote, label]) => verbatimString(atCount, quote, label)),
        ),
        ...quotes.map(([quote, label]) => normalString(quote, label)),
    ];
}

/**
 * Create a balanced-delimiter rule whose begin and end retain punctuation scopes.
 */
function delimitedContent(delimiter: 'braces' | 'brackets' | 'parens', begin: string, end: string, include: string) {
    return {
        begin,
        beginCaptures: { 0: { name: `punctuation.section.${delimiter}.begin.mira` } },
        end,
        endCaptures: { 0: { name: `punctuation.section.${delimiter}.end.mira` } },
        patterns: [{ include }],
    };
}

/**
 * Build the documentation-comment rules shared by source and doc grammars.
 */
function documentationRepository(sourceInclude = '#source') {
    return {
        documentation: {
            patterns: [
                { include: '#documentation-mirascript-fence' },
                { include: '#documentation-other-fence' },
                { include: '#documentation-line' },
                { include: '#documentation-fragment' },
            ],
        },
        'documentation-inline': {
            patterns: [
                {
                    name: `storage.type.class.documentation.mira`,
                    match: String.raw`@(param|returns)\b`,
                },
                {
                    name: `markup.bold.documentation.mira`,
                    match: String.raw`\*\*[^*]+\*\*`,
                },
                { name: `markup.italic.documentation.mira`, match: String.raw`\*[^*]+\*` },
                { name: `constant.character.escape.documentation.mira`, match: String.raw`\\\*` },
            ],
        },
        'documentation-line': {
            patterns: [
                {
                    match: String.raw`^(?!\s*\*/)(\s*\*\s?)(.*?)(?=\*/|$)`,
                    captures: {
                        1: { name: `comment.block.documentation.mira` },
                        2: {
                            name: `meta.documentation.inline.mira`,
                            patterns: [{ include: '#documentation-inline' }],
                        },
                    },
                },
            ],
        },
        'documentation-fragment': {
            patterns: [
                {
                    match: String.raw`(?!\*/)(.+?)(?=\*/|$)`,
                    captures: {
                        1: {
                            name: `meta.documentation.inline.mira`,
                            patterns: [{ include: '#documentation-inline' }],
                        },
                    },
                },
            ],
        },
        'documentation-mirascript-fence': {
            patterns: [
                {
                    name: `markup.fenced_code.block.mira`,
                    begin: documentationMiraFenceBegin,
                    beginCaptures: {
                        1: { name: `comment.block.documentation.mira` },
                        2: { name: `punctuation.definition.markdown.mira` },
                        3: { name: `fenced_code.block.language.mira` },
                    },
                    end: documentationFenceEnd,
                    endCaptures: {
                        1: { name: `comment.block.documentation.mira` },
                        2: { name: `punctuation.definition.markdown.mira` },
                    },
                    patterns: [
                        {
                            name: `meta.embedded.block.mira`,
                            contentName: `source.mira`,
                            begin: documentationMiraLine,
                            beginCaptures: {
                                1: { name: `comment.block.documentation.mira` },
                            },
                            while: documentationMiraLine,
                            whileCaptures: {
                                1: { name: `comment.block.documentation.mira` },
                            },
                            patterns: [{ include: sourceInclude }],
                        },
                        {
                            name: `comment.block.documentation.mira`,
                            match: String.raw`^(?!\s*\*/)(\s*\*\s?)`,
                        },
                    ],
                },
            ],
        },
        'documentation-other-fence': {
            patterns: [
                {
                    name: `markup.fenced_code.block.mira`,
                    contentName: `markup.raw.block.mira`,
                    begin: documentationOtherFenceBegin,
                    beginCaptures: {
                        1: { name: `comment.block.documentation.mira` },
                        2: { name: `punctuation.definition.markdown.mira` },
                        3: { name: `fenced_code.block.language.mira` },
                    },
                    end: documentationFenceEnd,
                    endCaptures: {
                        1: { name: `comment.block.documentation.mira` },
                        2: { name: `punctuation.definition.markdown.mira` },
                    },
                    patterns: [
                        {
                            name: `comment.block.documentation.mira`,
                            match: String.raw`^(?!\s*\*/)(\s*\*\s?)`,
                        },
                    ],
                },
            ],
        },
    };
}

/**
 * Build the shared repository used by source and template grammars.
 */
function sourceRepository(): LanguageRegistration['repository'] {
    /**
     * Create a boundary-aware TextMate keyword rule.
     */
    const keywordRule = (values: readonly string[], name: string) => ({
        name: `${name}.mira`,
        match: `${IDENTIFIER_START}(?:${alternatives(values)})${IDENTIFIER_END}`,
    });
    const keywordPatterns = [
        keywordRule(NUMERIC_KEYWORDS, 'constant.numeric.language'),
        keywordRule(constantKeywords, 'constant.language'),
        keywordRule(CONTROL_KEYWORDS, 'keyword.control'),
        keywordRule(wordOperators, 'keyword.operator.wordlike'),
        keywordRule(declarationKeywords, 'keyword.declaration'),
        keywordRule(moduleKeywords, 'keyword.module'),
        keywordRule(RESERVED_KEYWORDS, 'keyword.reserved'),
        keywordRule(languageVariables, 'variable.language'),
        keywordRule(otherKeywords, 'keyword.other'),
    ];

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
                { name: `comment.line.double-slash.mira`, match: '//.*$' },
                {
                    name: `comment.block.documentation.mira`,
                    begin: String.raw`/\*\*`,
                    end: String.raw`\*/`,
                },
                { name: `comment.block.mira`, begin: String.raw`/\*`, end: String.raw`\*/` },
            ],
        },
        ...documentationRepository(),
        'format-content': {
            patterns: [
                { name: `constant.character.escape.format.mira`, match: String.raw`\\.` },
                {
                    begin: String.raw`\[`,
                    end: String.raw`\]`,
                    name: `string.unquoted.format.character-class.mira`,
                    patterns: [{ name: `constant.character.escape.format.mira`, match: String.raw`\\.` }],
                },
                {
                    begin: String.raw`\(`,
                    end: String.raw`\)`,
                    name: `string.unquoted.format.group.mira`,
                    patterns: [{ include: '#format-content' }],
                },
            ],
        },
        strings: { patterns: stringPatterns() },
        'function-declarations': {
            patterns: [
                {
                    name: `meta.function.declaration.mira`,
                    begin: String.raw`${IDENTIFIER_START}(fn)(\s+)(${IDENTIFIER})(\s*)(\()`,
                    beginCaptures: {
                        1: { name: `keyword.declaration.function.mira` },
                        3: { name: `entity.name.function.mira` },
                        5: { name: `punctuation.section.parameters.begin.mira` },
                    },
                    end: String.raw`\)`,
                    endCaptures: { 0: { name: `punctuation.section.parameters.end.mira` } },
                    patterns: [{ include: '#parameters' }],
                },
                {
                    name: `meta.function.declaration.mira`,
                    match: String.raw`${IDENTIFIER_START}(fn)(\s+)(${IDENTIFIER})`,
                    captures: {
                        1: { name: `keyword.declaration.function.mira` },
                        3: { name: `entity.name.function.mira` },
                    },
                },
                {
                    name: `meta.function.expression.parameters.mira`,
                    begin: String.raw`${IDENTIFIER_START}(fn)(\s*)(\()`,
                    beginCaptures: {
                        1: { name: `keyword.declaration.function.mira` },
                        3: { name: `punctuation.section.parameters.begin.mira` },
                    },
                    end: String.raw`\)`,
                    endCaptures: { 0: { name: `punctuation.section.parameters.end.mira` } },
                    patterns: [{ include: '#parameters' }],
                },
            ],
        },
        parameters: {
            patterns: [
                {
                    match: String.raw`${IDENTIFIER_START}(mut)(\s+)(${IDENTIFIER})`,
                    captures: {
                        1: { name: `keyword.declaration.mutable.mira` },
                        3: { name: `variable.emphasis.mira` },
                    },
                },
                {
                    name: `keyword.declaration.mutable.mira`,
                    match: `${IDENTIFIER_START}mut${IDENTIFIER_END}`,
                },
                { name: `keyword.operator.spread.mira`, match: String.raw`\.\.` },
                { name: `variable.other.constant.emphasis.mira`, match: IDENTIFIER },
                { name: `punctuation.separator.parameter.mira`, match: ',' },
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
                    match: String.raw`${IDENTIFIER_START}(mod)(\s+)(${IDENTIFIER})`,
                    captures: {
                        1: { name: `keyword.module.mira` },
                        3: { name: `entity.name.namespace.mira` },
                    },
                },
            ],
        },
        'for-bindings': {
            patterns: [
                {
                    match: String.raw`${IDENTIFIER_START}(for)(\s+)(mut)(\s+)(${IDENTIFIER})(\s+)(in)${IDENTIFIER_END}`,
                    captures: {
                        1: { name: `keyword.control.loop.mira` },
                        3: { name: `keyword.declaration.mutable.mira` },
                        5: { name: `variable.other.readwrite.mira` },
                        7: { name: `keyword.control.loop.mira` },
                    },
                },
                {
                    match: String.raw`${IDENTIFIER_START}(for)(\s+)(${IDENTIFIER})(\s+)(in)${IDENTIFIER_END}`,
                    captures: {
                        1: { name: `keyword.control.loop.mira` },
                        3: { name: `variable.other.mira` },
                        5: { name: `keyword.control.loop.mira` },
                    },
                },
            ],
        },
        'record-properties': {
            patterns: [
                {
                    match: String.raw`(${IDENTIFIER})(\s*)(\??:)(?!:)`,
                    captures: {
                        1: { name: `variable.other.property.mira` },
                        3: { name: `punctuation.separator.key-value.mira` },
                    },
                },
            ],
        },
        'member-access': {
            patterns: [
                {
                    match: String.raw`(\.)(\s*)(\d+)`,
                    captures: {
                        1: { name: `punctuation.accessor.mira` },
                        3: { name: `variable.other.property.mira` },
                    },
                },
                {
                    match: `(\\.)(\\s*)(${IDENTIFIER})(\\s*)(!?)(?=\\s*(?:\\(|@*["'\`]))`,
                    captures: {
                        1: { name: `punctuation.accessor.mira` },
                        3: { name: `entity.name.function.member.mira` },
                        5: { name: `keyword.operator.non-null.mira` },
                    },
                },
                {
                    match: String.raw`(\.)(\s*)(${IDENTIFIER})`,
                    captures: {
                        1: { name: `punctuation.accessor.mira` },
                        3: { name: `variable.other.property.mira` },
                    },
                },
            ],
        },
        'function-calls': {
            patterns: [
                {
                    match: `(${IDENTIFIER})(\\s*)(!?)(?=\\s*(?:\\(|@*["'\`]))`,
                    captures: {
                        1: { name: `entity.name.function.mira` },
                        3: { name: `keyword.operator.non-null.mira` },
                    },
                },
            ],
        },
        'type-keyword': {
            patterns: [
                {
                    name: `keyword.operator.expression.mira`,
                    match: String.raw`${IDENTIFIER_START}type${IDENTIFIER_END}(?=\s*\(|\s+${IDENTIFIER})`,
                },
            ],
        },
        numbers: {
            patterns: [
                {
                    name: `constant.numeric.hex.mira`,
                    match: `0[xX][0-9A-Fa-f](?:[0-9A-Fa-f_]*[0-9A-Fa-f])?${IDENTIFIER_END}`,
                },
                {
                    name: `constant.numeric.octal.mira`,
                    match: `0[oO][0-7](?:[0-7_]*[0-7])?${IDENTIFIER_END}`,
                },
                {
                    name: `constant.numeric.binary.mira`,
                    match: `0[bB][01](?:[01_]*[01])?${IDENTIFIER_END}`,
                },
                {
                    name: `invalid.illegal.numeric.mira`,
                    match: String.raw`0[xXoObB]\p{XID_Continue}*`,
                },
                {
                    name: `constant.numeric.float.mira`,
                    match: String.raw`\d[\d_]*(?:\.[\d_]+)?(?:[eE][+-]?[\d_]*\d)?${IDENTIFIER_END}`,
                },
            ],
        },
        keywords: { patterns: keywordPatterns },
        identifiers: {
            patterns: [
                { name: `variable.other.constant.mira`, match: String.raw`@+\p{XID_Continue}+` },
                { name: `variable.other.mira`, match: IDENTIFIER },
            ],
        },
        operators: {
            patterns: [
                { name: `keyword.operator.spread.mira`, match: String.raw`\.\.<|\.\.` },
                { name: `keyword.operator.bind.mira`, match: '::' },
                { name: `keyword.operator.conditional.mira`, match: String.raw`\?:` },
                {
                    name: `keyword.operator.assignment.mira`,
                    match: String.raw`&&=|\|\|=|\?\?=|\+=|-=|\*=|/=|%=|\^=|=`,
                },
                {
                    name: `keyword.operator.comparison.mira`,
                    match: '===|!==|>=|<=|==|!=|=~|!~|>|<',
                },
                { name: `keyword.operator.logical.mira`, match: String.raw`&&|\|\||\?\?|!` },
                { name: `keyword.operator.arithmetic.mira`, match: String.raw`\+|-|\*|/|%|\^` },
            ],
        },
        punctuation: {
            patterns: [
                { name: `punctuation.section.braces.mira`, match: '[{}]' },
                { name: `punctuation.section.brackets.mira`, match: String.raw`[\[\]]` },
                { name: `punctuation.section.parens.mira`, match: '[()]' },
                { name: `punctuation.separator.mira`, match: '[,;:]' },
                { name: `punctuation.accessor.mira`, match: String.raw`\.` },
            ],
        },
        'braced-content': {
            patterns: [
                delimitedContent('braces', String.raw`\{`, String.raw`\}`, '#braced-content'),
                delimitedContent('parens', String.raw`\(`, String.raw`\)`, '#parenthesized-content'),
                delimitedContent('brackets', String.raw`\[`, String.raw`\]`, '#bracketed-content'),
                { include: '#source' },
            ],
        },
        'interpolation-expression-content': {
            patterns: [
                delimitedContent('parens', String.raw`\(`, String.raw`\)`, '#parenthesized-content'),
                delimitedContent('braces', String.raw`\{`, String.raw`\}`, '#braced-content'),
                delimitedContent('brackets', String.raw`\[`, String.raw`\]`, '#bracketed-content'),
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
                delimitedContent('parens', String.raw`\(`, String.raw`\)`, '#parenthesized-content'),
                delimitedContent('braces', String.raw`\{`, String.raw`\}`, '#braced-content'),
                delimitedContent('brackets', String.raw`\[`, String.raw`\]`, '#bracketed-content'),
                { include: '#source' },
            ],
        },
        'bracketed-content': {
            patterns: [
                delimitedContent('brackets', String.raw`\[`, String.raw`\]`, '#bracketed-content'),
                delimitedContent('braces', String.raw`\{`, String.raw`\}`, '#braced-content'),
                delimitedContent('parens', String.raw`\(`, String.raw`\)`, '#parenthesized-content'),
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
        ...mirascriptLanguage,
        patterns: [{ include: '#source' }],
        repository: sourceRepository(),
        injections: {
            [documentationInjectionSelector]: {
                patterns: [{ include: '#documentation' }],
            },
        },
    };
}

/**
 * Create the MiraScript template TextMate grammar.
 */
export function createMiraScriptTemplateGrammar(): LanguageRegistration {
    return {
        ...mirascriptTemplateLanguage,
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
        ...mirascriptDocLanguage,
        patterns: [{ include: '#doc' }],
        injections: {
            [documentationInjectionSelector]: {
                patterns: [{ include: '#documentation' }],
            },
        },
        repository: {
            ...documentationRepository('source.mira#source'),
            doc: {
                patterns: [
                    { include: '#tag' },
                    { include: '#global-value' },
                    { include: '#global-constants' },
                    { include: '#inline-parameter' },
                    { include: '#properties' },
                    { include: '#inline-label' },
                    { include: '#binding-value' },
                    { include: '#bindings' },
                    { include: '#function-signatures' },
                    { include: '#generic-types' },
                    { include: '#parameters' },
                    { include: '#return-type' },
                    { include: '#tag-comment' },
                    { include: 'source.mira' },
                ],
            },
            tag: {
                patterns: [
                    {
                        name: 'meta.documentation.tag.mira',
                        contentName: 'meta.documentation.tag.content.mira',
                        begin: `(<)(?=${DOC_TAG_CONTENT}>)`,
                        beginCaptures: { 1: { name: 'punctuation.definition.tag.begin.mira' } },
                        end: '(>)',
                        endCaptures: { 1: { name: 'punctuation.definition.tag.end.mira' } },
                        patterns: [{ include: '#tag-content' }],
                    },
                ],
            },
            'global-value': {
                patterns: [
                    {
                        name: 'meta.documentation.global-value.mira',
                        begin: String.raw`^(\x00)?(\(global\))(\s+)(${IDENTIFIER})(\s*)(=)`,
                        beginCaptures: {
                            2: { name: 'entity.name.label.mira' },
                            4: { name: 'variable.other.mira' },
                            6: { name: 'keyword.operator.assignment.mira' },
                        },
                        end: '$',
                        patterns: [{ include: '#doc-value' }],
                    },
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
                        match: String.raw`${IDENTIFIER_START}(let)(\s+)(mut)(\s+)(${IDENTIFIER})`,
                        captures: {
                            1: { name: 'keyword.declaration.variable.mira' },
                            3: { name: 'keyword.declaration.mutable.mira' },
                            5: { name: 'variable.other.readwrite.mira' },
                        },
                    },
                    {
                        match: String.raw`${IDENTIFIER_START}(let|const)(\s+)(${IDENTIFIER})`,
                        captures: {
                            1: { name: 'keyword.declaration.variable.mira' },
                            3: { name: 'variable.other.constant.mira' },
                        },
                    },
                ],
            },
            'binding-value': {
                patterns: [
                    {
                        begin: String.raw`${IDENTIFIER_START}(let)(\s+)(mut)(\s+)(${IDENTIFIER})(\s*)(=)`,
                        beginCaptures: {
                            1: { name: 'keyword.declaration.variable.mira' },
                            3: { name: 'keyword.declaration.mutable.mira' },
                            5: { name: 'variable.other.readwrite.mira' },
                            7: { name: 'keyword.operator.assignment.mira' },
                        },
                        end: '$',
                        patterns: [{ include: '#doc-value' }],
                    },
                    {
                        begin: String.raw`${IDENTIFIER_START}(let|const)(\s+)(${IDENTIFIER})(\s*)(=)`,
                        beginCaptures: {
                            1: { name: 'keyword.declaration.variable.mira' },
                            3: { name: 'variable.other.constant.mira' },
                            5: { name: 'keyword.operator.assignment.mira' },
                        },
                        end: '$',
                        patterns: [{ include: '#doc-value' }],
                    },
                ],
            },
            'function-signatures': {
                patterns: [
                    {
                        match: String.raw`${IDENTIFIER_START}(?:(pub)(\s+))?(fn)(\s+)(${IDENTIFIER})(?=\s*(?:<|\())`,
                        captures: {
                            1: { name: 'keyword.module.mira' },
                            3: { name: 'keyword.declaration.function.mira' },
                            5: { name: 'entity.name.function.mira' },
                        },
                    },
                ],
            },
            properties: {
                patterns: [
                    {
                        name: 'meta.documentation.field.mira',
                        begin: String.raw`^(\x00)?(?:(\(field\))(\s+))?(${DOC_FIELD_NAME})(\s*)(\??:)(?=[ \t]*/\*[ \t]*<extern[ \t]+(?:(?:async[ \t]+)?function\*?)(?:[ \t]+${DOC_TAG_NAME})?>[ \t]*\*/)`,
                        beginCaptures: {
                            2: { name: 'entity.name.label.mira' },
                            4: { name: 'entity.name.function.mira' },
                            6: { name: 'punctuation.separator.type.mira' },
                        },
                        end: '$',
                        patterns: [
                            { include: '#tag-comment' },
                            { include: '#tag' },
                            { include: '#doc-record-value' },
                            { include: '#doc-array-value' },
                            { include: '#type-context' },
                        ],
                    },
                    {
                        name: 'meta.documentation.field.mira',
                        begin: String.raw`^(\x00)?(?:(\(field\))(\s+))?(${DOC_FIELD_NAME})(\s*)(\??:)(?=[ \t]*/\*[ \t]*<extern[ \t]+class(?:[ \t]+${DOC_TAG_NAME})?>[ \t]*\*/)`,
                        beginCaptures: {
                            2: { name: 'entity.name.label.mira' },
                            4: { name: 'entity.name.type.mira' },
                            6: { name: 'punctuation.separator.type.mira' },
                        },
                        end: '$',
                        patterns: [
                            { include: '#tag-comment' },
                            { include: '#tag' },
                            { include: '#doc-record-value' },
                            { include: '#doc-array-value' },
                            { include: '#type-context' },
                        ],
                    },
                    {
                        name: 'meta.documentation.field.mira',
                        begin: String.raw`^(\x00)?(?:(\(field\))(\s+))?(${DOC_FIELD_NAME})(\s*)(\??:)`,
                        beginCaptures: {
                            2: { name: 'entity.name.label.mira' },
                            4: { name: 'variable.other.property.mira' },
                            6: { name: 'punctuation.separator.type.mira' },
                        },
                        end: '$',
                        patterns: [
                            { include: '#tag-comment' },
                            { include: '#tag' },
                            { include: '#doc-record-value' },
                            { include: '#doc-array-value' },
                            { include: '#type-context' },
                        ],
                    },
                ],
            },
            'doc-value': {
                patterns: [
                    { include: '#tag-properties' },
                    { include: '#tag-comment' },
                    { include: '#tag' },
                    { include: '#doc-record-value' },
                    { include: '#doc-array-value' },
                    { include: 'source.mira' },
                ],
            },
            'doc-record-value': {
                patterns: [
                    {
                        begin: String.raw`\(`,
                        beginCaptures: { 0: { name: 'punctuation.section.parens.begin.mira' } },
                        end: String.raw`\)`,
                        endCaptures: { 0: { name: 'punctuation.section.parens.end.mira' } },
                        patterns: [{ include: '#doc-value' }],
                    },
                ],
            },
            'doc-array-value': {
                patterns: [
                    {
                        begin: String.raw`\[`,
                        beginCaptures: { 0: { name: 'punctuation.section.brackets.begin.mira' } },
                        end: String.raw`\]`,
                        endCaptures: { 0: { name: 'punctuation.section.brackets.end.mira' } },
                        patterns: [{ include: '#doc-value' }],
                    },
                ],
            },
            'tag-properties': {
                patterns: [
                    {
                        match: String.raw`(${DOC_FIELD_NAME})(\s*)(:)(?=[ \t]*/\*[ \t]*<extern[ \t]+(?:(?:async[ \t]+)?function\*?)(?:[ \t]+${DOC_TAG_NAME})?>[ \t]*\*/)`,
                        captures: {
                            1: { name: 'entity.name.function.mira' },
                            3: { name: 'punctuation.separator.key-value.mira' },
                        },
                    },
                    {
                        match: String.raw`(${DOC_FIELD_NAME})(\s*)(:)(?=[ \t]*/\*[ \t]*<extern[ \t]+class(?:[ \t]+${DOC_TAG_NAME})?>[ \t]*\*/)`,
                        captures: {
                            1: { name: 'entity.name.type.mira' },
                            3: { name: 'punctuation.separator.key-value.mira' },
                        },
                    },
                ],
            },
            'tag-comment': {
                patterns: [
                    {
                        name: 'comment.block.mira',
                        contentName: 'meta.documentation.tag.mira',
                        begin: String.raw`(/\*)([ \t]*)(<)(?=${DOC_TAG_CONTENT}>[ \t]*\*/)`,
                        beginCaptures: {
                            1: { name: 'punctuation.definition.comment.begin.mira' },
                            3: { name: 'punctuation.definition.tag.begin.mira' },
                        },
                        end: String.raw`(>)([ \t]*)(\*/)`,
                        endCaptures: {
                            1: { name: 'punctuation.definition.tag.end.mira' },
                            3: { name: 'punctuation.definition.comment.end.mira' },
                        },
                        patterns: [{ include: '#tag-content' }],
                    },
                ],
            },
            'tag-content': {
                patterns: [
                    {
                        match: String.raw`(extern)([ \t]+)(?:(async)([ \t]+))?(function)(\*)?(?:([ \t]+)(${DOC_TAG_NAME}))?(?=>)`,
                        captures: {
                            1: { name: 'keyword.declaration.extern.mira' },
                            3: { name: 'keyword.js' },
                            5: { name: 'keyword.js' },
                            6: { name: 'keyword.operator.generator.js' },
                            8: { name: 'entity.name.function.js' },
                        },
                    },
                    {
                        match: String.raw`(extern)([ \t]+)(class)(?:([ \t]+)(${DOC_TAG_NAME}))?(?=>)`,
                        captures: {
                            1: { name: 'keyword.declaration.extern.mira' },
                            3: { name: 'keyword.js' },
                            5: { name: 'entity.name.type.js' },
                        },
                    },
                    {
                        match: String.raw`(extern)([ \t]+)(${DOC_TAG_NAME})(\()(${DOC_TAG_NUMBER})(\))(?=>)`,
                        captures: {
                            1: { name: 'keyword.declaration.extern.mira' },
                            3: { name: 'entity.name.type.js' },
                            4: { name: 'punctuation.section.parens.begin.mira' },
                            5: { name: 'constant.numeric.mira' },
                            6: { name: 'punctuation.section.parens.end.mira' },
                        },
                    },
                    {
                        match: String.raw`(extern)(?:([ \t]+)(${DOC_TAG_NAME}))?(?=>)`,
                        captures: {
                            1: { name: 'keyword.declaration.extern.mira' },
                            3: { name: 'entity.name.type.js' },
                        },
                    },
                    {
                        match: String.raw`(module)(?:([ \t]+)(${DOC_TAG_NAME}))?(?=>)`,
                        captures: {
                            1: { name: 'keyword.declaration.module.mira' },
                            3: { name: 'entity.name.namespace.mira' },
                        },
                    },
                    {
                        match: String.raw`(function)(?:([ \t]+)(${DOC_TAG_NAME}))?(?=>)`,
                        captures: {
                            1: { name: 'keyword.declaration.function.mira' },
                            3: { name: 'entity.name.function.mira' },
                        },
                    },
                ],
            },
            'reflection-type': {
                patterns: [
                    {
                        match: String.raw`${IDENTIFIER_START}(type)(\s*)(\()(\s*)(${IDENTIFIER})(\s*)(\))`,
                        captures: {
                            1: { name: 'support.type.type.mira' },
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
                        begin: String.raw`${IDENTIFIER_START}(fn)${IDENTIFIER_END}(?=\s*(?:<|\())`,
                        beginCaptures: { 1: { name: 'support.type.function.mira' } },
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
                                    5: { name: 'variable.emphasis.mira' },
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
                            { name: 'variable.emphasis.mira', match: IDENTIFIER },
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
                        match: `${IDENTIFIER_START}(?:${alternatives(BUILT_IN_TYPES)})${IDENTIFIER_END}`,
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
