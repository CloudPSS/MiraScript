import * as monaco from 'monaco-editor';

const REG_IDENTIFIER = /(?:_+|@+|\$+|\p{XID_Start}+)\p{XID_Continue}*/u;
const REG_HEX = /0[xX][a-fA-F0-9_]+/u;
const REG_OCT = /0[oO][0-7_]+/u;
const REG_BIN = /0[bB][01_]+/u;
const REG_ORDINAL =
    /(?:0|[1-9]\d{0,8}|1\d{9}|20\d{8}|21[0-3]\d{7}|214[0-6]\d{6}|2147[0-3]\d{5}|21474[0-7]\d{4}|214748[0-2]\d{3}|2147483[0-5]\d{2}|21474836[0-3]\d|214748364[0-7])/u;
const REG_NUMBER = /(?<!\.\s*)\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/u;
const REG_WHITESPACE = /[ \t\v\f\r\n]/u;

monaco.languages.setLanguageConfiguration('mirascript', {
    comments: {
        lineComment: '//',
        blockComment: ['/*', '*/'],
    },
    brackets: [
        ['(', ')'],
        ['[', ']'],
        ['{', '}'],
    ],
    wordPattern: new RegExp(
        `(${REG_IDENTIFIER.source}|${REG_HEX.source}|${REG_OCT.source}|${REG_BIN.source}|${REG_NUMBER.source}|\\d+)`,
        'gu',
    ),
    autoClosingPairs: [
        {
            open: '{',
            close: '}',
        },
        {
            open: '[',
            close: ']',
        },
        {
            open: '(',
            close: ')',
        },
        {
            open: '"',
            close: '"',
            notIn: ['string', 'string_double'],
        },
        {
            open: "'",
            close: "'",
            notIn: ['string', 'string_single'],
        },
        {
            open: '`',
            close: '`',
            notIn: ['string', 'string_backtick'],
        },
    ],
    autoCloseBefore: ')}];, ',
});

const MAX_VERBATIM_LENGTH = 16;

monaco.languages.setMonarchTokensProvider('mirascript', {
    ignoreCase: false,
    unicode: true,
    includeLF: false,
    brackets: [
        { open: '{', close: '}', token: 'delimiter.curly' },
        { open: '[', close: ']', token: 'delimiter.square' },
        { open: '(', close: ')', token: 'delimiter.parenthesis' },
    ],
    // defaultToken: 'invalid',
    tokenizer: {
        root: [[/[[\](){}]/gu, '@brackets'], { include: '@common' }],
        common: [
            { include: '@whitespace' },
            { include: '@string' },
            { include: '@identifier' },
            [
                new RegExp(`(\\.)(${REG_WHITESPACE.source}*)(\\d+)`, 'gu'),
                [
                    'operator.dot',
                    '',
                    {
                        cases: {
                            [REG_ORDINAL.source]: 'number.ordinal',
                            '@default': 'number.float',
                        },
                    },
                ],
            ],
            [REG_OCT, 'number.octal'],
            [REG_BIN, 'number.binary'],
            [REG_HEX, 'number.hexadecimal'],
            [
                REG_NUMBER,
                {
                    cases: {
                        [REG_ORDINAL.source]: 'number.ordinal',
                        '@default': 'number.float',
                    },
                },
            ],
            { include: '@operator' },
            [REG_ORDINAL, 'number.ordinal'],
        ],
        identifier: [
            [
                REG_IDENTIFIER,
                {
                    cases: {
                        '@keywords': { token: 'keyword.$0' },
                        '@reservedKeywords': { token: 'keyword.reserved.$0' },
                        '@numericConstants': { token: 'number.constant.$0' },
                        '@constants': { token: 'keyword.constant.$0' },
                        '@default': { token: 'identifier.$0' },
                    },
                },
            ],
        ],
        operator: [
            [/(!)(==)/gu, ['operator.exclamation', 'operator.equal-equal']],
            [/(!~=)/gu, 'operator.not-tilde-equal'],
            [/(!=)/gu, 'operator.not-equal'],
            [/(~=)/gu, 'operator.tilde-equal'],
            [/(==)/gu, 'operator.equal-equal'],
            [/(>=)/gu, 'operator.greater-equal'],
            [/(<=)/gu, 'operator.less-equal'],
            [/>/gu, 'operator.greater'],
            [/</gu, 'operator.less'],
            [/&&=/gu, 'operator.logical-and-equal'],
            [/&&/gu, 'operator.logical-and'],
            [/\|\|=/gu, 'operator.logical-or-equal'],
            [/\|\|/gu, 'operator.logical-or'],
            [/\?\?=/gu, 'operator.null-coalescing-equal'],
            [/\?\?/gu, 'operator.null-coalescing'],
            [/(\.{2}<)/gu, 'operator.half-open-range'],
            [/(\.{2})/gu, 'operator.spread-range'],
            [/(\+=)/gu, 'operator.plus-equal'],
            [/(\+)/gu, 'operator.plus'],
            [/(-=)/gu, 'operator.minus-equal'],
            [/(->)/gu, 'operator.arrow'],
            [/(-)/gu, 'operator.minus'],
            [/(\*=)/gu, 'operator.asterisk-equal'],
            [/(\*)/gu, 'operator.asterisk'],
            [/(\/=)/gu, 'operator.slash-equal'],
            [/(\/)/gu, 'operator.slash'],
            [/(%=)/gu, 'operator.percent-equal'],
            [/(%)/gu, 'operator.percent'],
            [/(\^=)/gu, 'operator.caret-equal'],
            [/(\^)/gu, 'operator.caret'],
            [/\?:/gu, 'delimiter.question-colon'],
            [/!:/gu, 'delimiter.exclamation-colon'],
            [/::/gu, 'operator.colon-colon'],
            [/!/gu, 'operator.exclamation'],
            [/=/gu, 'operator.equal'],
            [/\./gu, 'operator.dot'],
            [/:/gu, 'delimiter.colon'],
            [/,/gu, 'delimiter.comma'],
            [/;/gu, 'delimiter.semicolon'],
        ],
        whitespace: [
            [new RegExp(REG_WHITESPACE.source + '+', 'ug'), ''],
            [/\/\/.+$/gu, 'comment'],
            [/\/\*/gu, 'comment', '@block_comment'],
        ],
        block_comment: [
            [/\*\//gu, 'comment', '@pop'],
            [/[^*]+/, 'comment'],
            [/\*/, 'comment'],
        ],
        string: [
            [/(@*)"/gu, { token: 'string.quote.open.$1', next: '@string_double.$1', bracket: '@open' }],
            [/(@*)'/gu, { token: 'string.quote.open.$1', next: '@string_single.$1', bracket: '@open' }],
            [/(@*)`/gu, { token: 'string.quote.open.$1', next: '@string_backtick.$1', bracket: '@open' }],
        ],
        string_single: [
            [/[^'\\$]+/gu, 'string'],
            { include: '@string_escape' },
            [/(?=\$)/gu, '', '@string_interpolation.$S2'],
            [
                /'(@*)/gu,
                {
                    cases: {
                        '$S2==$1': { token: 'string.quote.close.$1', next: '@pop', bracket: '@close' },
                        '@default': 'string',
                    },
                },
            ],
        ],
        string_double: [
            [/[^"\\$]+/gu, 'string'],
            { include: '@string_escape' },
            [/(?=\$)/gu, '', '@string_interpolation.$S2'],
            [
                /"(@*)/gu,
                {
                    cases: {
                        '$S2==$1': { token: 'string.quote.close.$1', next: '@pop', bracket: '@close' },
                        '@default': 'string',
                    },
                },
            ],
        ],
        string_backtick: [
            [/[^`\\$]+/gu, 'string'],
            { include: '@string_escape' },
            [/(?=\$)/gu, '', '@string_interpolation.$S2'],
            [
                /`(@*)/gu,
                {
                    cases: {
                        '$S2==$1': { token: 'string.quote.close.$1', next: '@pop', bracket: '@close' },
                        '@default': 'string',
                    },
                },
            ],
        ],
        string_escape: [
            [/\\\\/gu, 'string.escape.backslash'],
            [/\\'/gu, 'string.escape.apostrophe'],
            [/\\"/gu, 'string.escape.quote'],
            [/\\`/gu, 'string.escape.backtick'],
            [/\\\$/gu, 'string.escape.dollar'],
            [/\\([rntbfv0])/gu, { token: 'string.escape.$1' }],
            [/\\u\{([0-9a-fA-F]+)\}/gu, { token: 'string.escape.unicode.$1' }],
            [/\\x([0-9a-fA-F]{2})/gu, { token: 'string.escape.ascii.$1' }],
            [/\\./gu, { token: 'string.escape.invalid' }],
        ],
        ...Object.fromEntries(
            Array.from({ length: MAX_VERBATIM_LENGTH }).map((_, i) => {
                const dollarCount = i === 0 ? 1 : i;
                const dollarRegex = `\\\${${dollarCount}}`;
                const name = `string_interpolation.${'@'.repeat(i)}`;
                return [
                    name,
                    [
                        [
                            new RegExp(`(${dollarRegex})(?=${REG_IDENTIFIER.source})`, 'gu'),
                            `string.interpolation.$1`,
                            '@string_interpolation_identifier',
                        ],
                        [
                            new RegExp(`(${dollarRegex})(\\{)`, 'gu'),
                            [
                                'string.interpolation',
                                {
                                    token: '@brackets',
                                    next: '@string_interpolation_expression',
                                },
                            ],
                        ],
                        [new RegExp(`\\\${0,${dollarCount}}`, 'gu'), 'string', '@pop'],
                        ['', '', '@pop'],
                    ],
                ];
            }),
        ),
        string_interpolation: [[/\$*/gu, 'string', '@pop']],
        string_interpolation_identifier: [
            [
                REG_IDENTIFIER,
                {
                    cases: {
                        '@keywords': { token: 'keyword.$0', next: '@pop' },
                        '@reservedKeywords': { token: 'keyword.$0', next: '@pop' },
                        '@numericConstants': { token: 'number.constant.$0', next: '@pop' },
                        '@constants': { token: 'keyword.constant.$0', next: '@pop' },
                        '@default': { token: 'identifier.$0', next: '@pop' },
                    },
                },
            ],
            ['', '', '@pop'],
        ],
        string_interpolation_expression: [
            [/\{/gu, { token: '@brackets', next: '@push' }],
            [/\}/gu, { token: '@brackets', next: '@pop' }],
            { include: '@common' },
        ],
    },
    keywords: [
        // pseudo variable
        '_',
        'global',
        // operators
        'in',
        'is',
        'and',
        'or',
        'not',
        // pseudo function
        'type',
        // control flow
        'if',
        'else',
        'match',
        'case',
        'for',
        'while',
        'loop',
        'break',
        'continue',
        'return',
        // declaration
        'fn',
        'let',
        'mut',
    ],
    reservedKeywords: [
        // declaration reserved
        'op',
        'where',
        // module reserved
        'import',
        'export',
        // algebraic effects reserved
        'effect',
        'try',
        'handle',
        'finally',
        'perform',
        'resume',
    ],
    constants: ['true', 'false', 'nil'],
    numericConstants: ['nan', 'inf'],
});
