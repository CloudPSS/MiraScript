import type { languages } from 'monaco-editor';
import {
    REG_WHITESPACE,
    REG_ORDINAL,
    REG_OCT,
    REG_BIN,
    REG_HEX,
    REG_NUMBER,
    REG_IDENTIFIER,
    DOC_HEADER,
    MAX_VERBATIM_LENGTH,
    constantKeywords,
    controlKeywords,
    keywords,
    numericKeywords,
} from '../constants';
import type { Monaco, IDisposable } from '../index.js';

/** 匹配 identifier */
function identifierCases(
    data?: Partial<languages.IExpandedMonarchLanguageAction>,
    defaultToken = 'variable',
): Record<string, languages.IExpandedMonarchLanguageAction> {
    return {
        '@numericKeywords': { ...data, token: `constant.numeric` },
        '@constantKeywords': { ...data, token: `constant.language` },
        '@controlKeywords': { ...data, token: `keyword.control` },
        '@keywords': { ...data, token: `keyword` },
        '[@]+.*': { ...data, token: `variable.other.constant` },
        '~it': { ...data, token: `variable.other.constant.emphasis` },
        '@default': { ...data, token: defaultToken },
    };
}

/** 生成 TokensProvider */
function getTokensProvider(mode: string): languages.IMonarchLanguage {
    return {
        ignoreCase: false,
        unicode: true,
        includeLF: false,
        brackets: [
            { open: '{', close: '}', token: 'delimiter.curly' },
            { open: '[', close: ']', token: 'delimiter.square' },
            { open: '(', close: ')', token: 'delimiter.parenthesis' },
        ],
        defaultToken: 'invalid',

        whitespace: REG_WHITESPACE,
        identifier: REG_IDENTIFIER,
        docHeader: DOC_HEADER,

        keywords: keywords(),
        controlKeywords: controlKeywords(),
        constantKeywords: constantKeywords(),
        numericKeywords: numericKeywords(),

        start: mode === 'template' ? 'root_template' : 'root',
        tokenPostfix: '.mirascript',
        tokenizer: {
            root: [
                [/\0|\/\*\*@docHeader\*\*\/$/, 'comment.doc', '@doc_mode'],
                [/[[\](){}]/, '@brackets'],
                { include: '@common' },
            ],
            root_template: [
                [/[^$]+/, 'string'],
                [/(?=\$)/, '', '@string_interpolation.$S3'],
                [/[$]/, 'string'],
            ],
            common: [
                [/(@identifier)(@whitespace*)(\??:)(?!:)/, ['support.type.property-name', '', 'delimiter']],
                [
                    /(fn)(@whitespace+)(@identifier)(@whitespace*)($|[({])/,
                    [
                        'keyword',
                        '',
                        { cases: identifierCases(undefined, 'entity.name.function') },
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
                    /(\.)(@whitespace*)(\d+)/,
                    [
                        'delimiter',
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
                    /(\.)(@whitespace*)(@identifier)(@whitespace*)(\()/,
                    ['delimiter', '', 'entity.name.function', '', '@brackets'],
                ],
                [/(\.)(@whitespace*)(@identifier)/, ['delimiter', '', 'variable']],
                [
                    /(@identifier)(@whitespace*)(\()/,
                    [
                        {
                            cases: identifierCases(undefined, `entity.name.function`),
                        },
                        '',
                        '@brackets',
                    ],
                ],
                { include: '@whitespace' },
                { include: '@string' },
                [/(@identifier)/, { cases: identifierCases() }],
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
                [/[-+=/~?:;,.!@$%^&*<>]/, 'delimiter'],
                [REG_ORDINAL, 'number.ordinal'],
            ],
            whitespace: [
                [/(@whitespace)+/, ''],
                [/\/\/.*$/, 'comment.line'],
                [/\/\*/, 'comment.block', '@block_comment'],
            ],
            block_comment: [
                [/\*\//, { token: 'comment.block', next: '@pop' }],
                [/[^*]+/, { token: 'comment.block' }],
                [/\*/, { token: 'comment.block' }],
            ],
            string: [
                [/["'`]/, { token: 'string.quote.open', next: '@string_normal.$#', bracket: '@open' }],
                [
                    /(@+)(["'`])/,
                    { token: 'string.quote.open.$2$1.raw', next: '@string_verbatim.$2$1.$1', bracket: '@open' },
                ],
            ],
            string_normal: [
                [/[^'"`\\$]+/, 'string'],
                { include: '@string_escape' },
                [/(?=\$)/, '', '@string_interpolation.'],
                [
                    /['"`]/,
                    {
                        cases: {
                            '$S2==$#': { token: 'string.quote.close', next: '@pop', bracket: '@close' },
                            '@default': 'string',
                        },
                    },
                ],
            ],
            string_verbatim: [
                [/[^'"`$]+/, 'string'],
                [/(?=\$)/, '', '@string_interpolation.$S3'],
                [
                    /(['"`]@+)/,
                    {
                        cases: {
                            '$S2==$#': { token: 'string.quote.close.raw.$#', next: '@pop', bracket: '@close' },
                            '@default': 'string',
                        },
                    },
                ],
                [/['"`$]/, 'string'],
            ],
            string_escape: [
                [/\\([\\'"`$rntbfv0])/, 'string.escape'],
                [/\\u\{([0-9a-fA-F]+)\}/, 'string.escape.unicode'],
                [/\\x([0-9a-fA-F]{2})/, 'string.escape.ascii'],
                [/\\./, { token: 'string.escape.invalid' }],
            ],
            ...Object.fromEntries(
                Array.from({ length: MAX_VERBATIM_LENGTH }).map((_, i) => {
                    const dollarCount = i === 0 ? 1 : i;
                    const dollarRegex = `\\\${${dollarCount}}`;
                    return [
                        `string_interpolation.${'@'.repeat(i)}`,
                        [
                            [
                                `(${dollarRegex})(${REG_IDENTIFIER.source})`,
                                ['punctuation.section.embedded', { cases: identifierCases({ next: '@pop' }) }],
                            ],
                            [
                                `(${dollarRegex}\\{)`,
                                {
                                    token: 'punctuation.section.embedded',
                                    bracket: '@open',
                                    next: '@string_interpolation_expression',
                                },
                            ],
                            [`\\\${0,${dollarCount}}`, 'string', '@pop'],
                            ['', '', '@pop'],
                        ],
                    ];
                }),
            ),
            string_interpolation: [[/\$*/, 'string', '@pop']],
            string_interpolation_expression: [
                [/\{/, { token: '@brackets', next: '@push' }],
                [/\}/, { token: '@brackets', next: '@pop' }],
                [/[[\]()]/, '@brackets'],
                { include: '@common' },
            ],

            doc_mode: [
                [/(fn)(@whitespace+)(@identifier)$/, ['keyword.fn.doc', '', 'entity.name.function.doc']],
                [/fn/, 'keyword.fn.doc', '@fn_doc'],
                [
                    /(\(parameter\))(@whitespace+)(..|)(mut)(@whitespace+)(@identifier)/,
                    ['entity.name.label', '', 'delimiter', 'keyword.mut', '', 'variable.emphasis'],
                ],
                [
                    /(\(parameter\))(@whitespace+)(..|)(@identifier)/,
                    ['entity.name.label', '', 'delimiter', 'variable.other.constant.emphasis'],
                ],
                [/(\(@identifier\))(@whitespace+)/, ['entity.name.label', '']],
                [/(let|const)(@whitespace+)(@identifier)/, [{ token: 'keyword.$1' }, '', 'variable.other.constant']],
                [
                    /(let)(@whitespace+)(mut)(@whitespace+)(@identifier)/,
                    [{ token: 'keyword.$1' }, '', 'keyword.mut', '', 'variable'],
                ],
                { include: '@common' },
            ],
            fn_doc: [
                [/(@identifier)(\()/, ['entity.name.function.doc', '@brackets']],
                [/@whitespace+/, ''],
                [
                    /(\.\.|)(@identifier)(\s*)(:|,|\))/,
                    [
                        'delimiter',
                        'variable.other.constant.emphasis.doc',
                        '',
                        {
                            cases: {
                                ':': { token: 'delimiter', next: '@type_doc' },
                                '\\)': '@brackets',
                                '@default': 'delimiter',
                            },
                        },
                    ],
                ],
                [/[()]/, '@brackets'],
                [/(->)/, 'delimiter', '@type_doc'],
                [/;/, 'delimiter', '@pop'],
            ],
            type_doc: [
                { include: '@type_doc_inner' },
                [/,/, 'delimiter', '@pop'],
                [/;/, { token: 'delimiter', next: '@pop', goBack: 1 }],
            ],
            type_doc_inner: [
                [/fn\b/, 'type', '@fn_doc'],
                [
                    /(type)(\()(@identifier)(\))/,
                    ['type', '@brackets', 'variable.other.constant.emphasis.doc', '@brackets'],
                ],
                [/@identifier/, 'type'],
                [/[[(]/, '@brackets', '@type_doc_inner'],
                [/[\])]/, '@brackets', '@pop'],
                [/[&|]/, 'delimiter'],
                [/@whitespace+/, ''],
            ],
        },
    };
}

/** 注册 Mirascript 的 TokensProvider */
export function registerMiraScriptTokensProvider(monaco: Monaco): IDisposable[] {
    return [
        monaco.languages.setMonarchTokensProvider('mirascript', getTokensProvider('script')),
        monaco.languages.setMonarchTokensProvider('mirascript-template', getTokensProvider('template')),
    ];
}
