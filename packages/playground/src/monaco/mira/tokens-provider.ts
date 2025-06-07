import { languages } from '@private/monaco-editor';
import {
    REG_WHITESPACE,
    REG_ORDINAL,
    REG_OCT,
    REG_BIN,
    REG_HEX,
    REG_NUMBER,
    REG_IDENTIFIER,
    DOC_HEADER,
} from './constants';
import { callWorker } from './worker-helper.js';

const MAX_VERBATIM_LENGTH = 16;

/** 匹配 identifier */
function identifierCases(
    capture = 0,
    data?: Partial<languages.IExpandedMonarchLanguageAction>,
    defaultToken = 'variable',
): Record<string, languages.IExpandedMonarchLanguageAction> {
    return {
        '@numericKeywords': { ...data, token: `constant.numeric.$${capture}` },
        '@constantKeywords': { ...data, token: `constant.language.$${capture}` },
        '@controlKeywords': { ...data, token: `keyword.control.$${capture}` },
        '@keywords': { ...data, token: `keyword.$${capture}` },
        '[@]+.*': { ...data, token: `variable.other.constant` },
        '~it': { ...data, token: `variable.other.constant.emphasis` },
        '@default': { ...data, token: defaultToken },
    };
}

/** 生成 TokensProvider */
async function getTokensProvider(): Promise<languages.IMonarchLanguage> {
    return {
        ignoreCase: false,
        unicode: true,
        includeLF: false,
        brackets: [
            { open: '{', close: '}', token: 'delimiter.curly' },
            { open: '[', close: ']', token: 'delimiter.square' },
            { open: '(', close: ')', token: 'delimiter.parenthesis' },
        ],
        //defaultToken: 'invalid',

        whitespace: REG_WHITESPACE,
        identifier: REG_IDENTIFIER,
        docHeader: new RegExp(DOC_HEADER, 'u'),

        keywords: await callWorker('keywords'),
        controlKeywords: await callWorker('control_keywords'),
        constantKeywords: await callWorker('constant_keywords'),
        numericKeywords: await callWorker('numeric_keywords'),

        tokenizer: {
            root: [
                [/\0|\/\*\*@docHeader\*\*\/$/gu, 'comment.doc', '@doc_mode'],
                [/[[\](){}]/gu, '@brackets'],
                { include: '@common' },
            ],
            common: [
                [
                    /(@identifier)(@whitespace*)(\??:)(?!:)/gu,
                    [
                        { token: 'support.type.property-name.$1' },
                        '',
                        {
                            cases: {
                                '\\?:': 'operator.question-colon',
                                ':': 'operator.colon',
                            },
                        },
                    ],
                ],
                [
                    /(fn)(@whitespace+)(@identifier)(@whitespace*)($|[({])/gu,
                    [
                        { token: 'keyword.$1' },
                        '',
                        { cases: identifierCases(3, undefined, 'entity.name.function') },
                        '',
                        {
                            cases: {
                                '~[({]': '@brackets',
                                '@default': '',
                            },
                        },
                    ],
                ],
                [
                    /(\.)(@whitespace*)(\d+)/gu,
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
                [
                    /(\.)(@whitespace*)(@identifier)(@whitespace*)(\()/gu,
                    ['operator.dot', '', { token: 'entity.name.function' }, '', '@brackets'],
                ],
                [/(\.)(@whitespace*)(@identifier)/gu, ['operator.dot', '', { token: 'variable.$3' }]],
                [
                    /(@identifier)(@whitespace*)(\()/gu,
                    [
                        {
                            cases: identifierCases(1, undefined, `entity.name.function`),
                        },
                        '',
                        '@brackets',
                    ],
                ],
                { include: '@whitespace' },
                { include: '@string' },
                { include: '@identifier' },
                [REG_OCT, 'number.octal'],
                [REG_BIN, 'number.binary'],
                [REG_HEX, 'number.hex'],
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
            identifier: [[/(@identifier)/gu, { cases: identifierCases() }]],
            operator: [
                [/(!)(==)/gu, ['operator.exclamation', 'operator.equal-equal']],
                [/(!)(::)/gu, ['operator.exclamation', 'operator.colon-colon']],
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
                [/(\.\.<)/gu, 'operator.half-open-range'],
                [/(\.\.)/gu, 'operator.spread-range'],
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
                [/::/gu, 'operator.colon-colon'],
                [/!/gu, 'operator.exclamation'],
                [/=/gu, 'operator.equal'],
                [/\./gu, 'operator.dot'],
                [/:/gu, 'delimiter.colon'],
                [/,/gu, 'delimiter.comma'],
                [/;/gu, 'delimiter.semicolon'],
            ],
            whitespace: [
                [/(@whitespace)+/gu, ''],
                [/\/\/.*$/gu, 'comment.line'],
                [/\/\*/gu, 'comment.block', '@block_comment'],
            ],
            block_comment: [
                [/\*\//gu, { token: 'comment.block', next: '@pop' }],
                [/[^*]+/, { token: 'comment.block' }],
                [/\*/, { token: 'comment.block' }],
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
                                { token: `punctuation.section.embedded.$1`, next: '@string_interpolation_identifier' },
                            ],
                            [
                                new RegExp(`(${dollarRegex})(\\{)`, 'gu'),
                                [
                                    { token: 'punctuation.section.embedded.$1' },
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
                [REG_IDENTIFIER, { cases: identifierCases(0, { next: '@pop' }) }],
                ['', '', '@pop'],
            ],
            string_interpolation_expression: [
                [/\{/gu, { token: '@brackets', next: '@push' }],
                [/\}/gu, { token: '@brackets', next: '@pop' }],
                [/[[\]()]/gu, '@brackets'],
                { include: '@common' },
            ],

            doc_mode: [
                [/(fn)(@whitespace+)(@identifier)$/g, ['keyword.fn.doc', '', 'entity.name.function.doc']],
                [/fn/g, 'keyword.fn.doc', '@fn_doc'],
                [
                    /(\(parameter\))(@whitespace+)(..|)(mut)(@whitespace+)(@identifier)/g,
                    ['entity.name.label', '', 'operator.spread-range', 'keyword.mut', '', 'variable.emphasis'],
                ],
                [
                    /(\(parameter\))(@whitespace+)(..|)(@identifier)/g,
                    ['entity.name.label', '', 'operator.spread-range', 'variable.other.constant.emphasis'],
                ],
                [/(\(@identifier\))/g, 'entity.name.label'],
                [
                    /(let|const)(@whitespace+)(@identifier)/g,
                    [{ token: 'keyword.$1' }, '', { cases: identifierCases(3, undefined, 'variable.other.constant') }],
                ],
                [
                    /(let)(@whitespace+)(mut)(@whitespace+)(@identifier)/g,
                    [
                        { token: 'keyword.$1' },
                        '',
                        'keyword.mut',
                        '',
                        { cases: identifierCases(3, undefined, 'variable') },
                    ],
                ],
                { include: '@common' },
            ],
            fn_doc: [
                [/(@identifier)(\()/gu, ['entity.name.function.doc', '@brackets']],
                [/@whitespace+/, ''],
                [
                    /(@identifier)(\s*)(:)/gu,
                    ['variable.other.constant.emphasis.doc', '', { token: 'operator.colon', next: '@type_doc' }],
                ],
                [
                    /(\.\.|)(@identifier)(\s*)(,)/gu,
                    ['operator.spread-range', 'variable.other.constant.emphasis.doc', '', 'operator.comma'],
                ],
                [/[()]/gu, '@brackets'],
                [/(->)/gu, 'operator.arrow', '@type_doc'],
            ],
            type_doc: [{ include: '@type_doc_inner' }, [/,/, 'operator.comma', '@pop']],
            type_doc_inner: [
                [/fn\b/gu, 'type', '@fn_doc'],
                [
                    /(type)(\()(@identifier)(\))/gu,
                    ['type', '@brackets', 'variable.other.constant.emphasis.doc', '@brackets'],
                ],
                [/@identifier/, 'type'],
                [/[[(]/, '@brackets', '@type_doc_inner'],
                [/[\])]/, '@brackets', '@pop'],
                [/@whitespace+/, ''],
            ],
        },
    };
}

void getTokensProvider().then((tokensProvider) => {
    languages.setMonarchTokensProvider('mirascript', tokensProvider);
});
