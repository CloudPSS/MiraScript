import { languages, type IDisposable } from '../monaco-api.js';
import {
    REG_WHITESPACE,
    REG_ORDINAL,
    REG_OCT,
    REG_BIN,
    REG_HEX,
    REG_NUMBER,
    REG_IDENTIFIER,
    MAX_VERBATIM_LENGTH,
    constantKeywords,
    controlKeywords,
    keywords,
    numericKeywords,
} from '../constants.js';
import { DefaultVmContext } from '@mirascript/mirascript/subtle';
import { isVmModule } from '@mirascript/mirascript';

const moduleNames = [...DefaultVmContext.keys()].filter(
    (name) => DefaultVmContext.has(name) && isVmModule(DefaultVmContext.get(name)),
);

/** 匹配 identifier */
function identifierCases(
    data?: Partial<languages.IExpandedMonarchLanguageAction>,
    defaultToken = 'variable',
): Record<string, languages.IExpandedMonarchLanguageAction> {
    return {
        '@numericKeywords': { ...data, token: `constant.numeric` },
        '@constantKeywords': { ...data, token: `constant.language` },
        '@controlKeywords': { ...data, token: `keyword.flow` },
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

        keywords: keywords(),
        controlKeywords: controlKeywords(),
        constantKeywords: constantKeywords(),
        numericKeywords: numericKeywords(),

        inlineDocParam: /\(parameter(?: pattern)?\)/,
        inlineDocMod: ['local', 'global', 'field', 'module'].join('|'),

        start: mode === 'template' ? 'root_template' : mode === 'doc' ? 'root_doc' : 'root',
        tokenPostfix: '.mirascript',
        tokenizer: {
            root: [[/[[\](){}]/, '@brackets'], { include: '@common' }],
            root_template: [
                [/[^$]+/, 'string'],
                [/(?=\$)/, '', '@string_interpolation.$S3'],
                [/[$]/, 'string'],
            ],
            common: [
                [/(@identifier)(@whitespace*)(\??:)(?!:)/, ['variable.other.property', '', 'delimiter']],
                [
                    /(fn)(@whitespace+)(@identifier)(?=$|@whitespace|[[({,;])/,
                    ['keyword', '', { cases: identifierCases(undefined, 'entity.name.function') }],
                ],
                [
                    /(for)(@whitespace+)(mut)(@whitespace+)(@identifier)(@whitespace+)(in)/,
                    ['keyword.flow', '', 'keyword', '', { cases: identifierCases() }, '', 'keyword.flow'],
                ],
                [
                    /(for)(@whitespace+)(@identifier)(@whitespace+)(in)/,
                    ['keyword.flow', '', { cases: identifierCases() }, '', 'keyword.flow'],
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
                [String.raw`\b(${moduleNames.join('|')})(@whitespace*(?=!?\.))`, ['type', '']],
                [
                    /(\.)(@whitespace*)(@identifier)(@whitespace*)(!?)(@whitespace*(?=\())/,
                    ['delimiter', '', 'entity.name.function', '', 'delimiter', ''],
                ],
                [/(\.)(@whitespace*)(@identifier)/, ['delimiter', '', 'variable']],
                [
                    /(@identifier)(@whitespace*)(!?)(@whitespace*(?=\())/,
                    [
                        {
                            cases: identifierCases(undefined, `entity.name.function`),
                        },
                        '',
                        'delimiter',
                        '',
                    ],
                ],
                { include: '@whitespace' },
                { include: '@string' },
                [/(@identifier)/, { cases: identifierCases() }],
                [
                    /0[xobXOB]\p{XID_Continue}*/u,
                    {
                        cases: {
                            [REG_OCT.source]: 'number.octal',
                            [REG_BIN.source]: 'number.binary',
                            [REG_HEX.source]: 'number.hex',
                            '@default': 'number.invalid',
                        },
                    },
                ],
                [
                    REG_NUMBER,
                    {
                        cases: {
                            [REG_ORDINAL.source]: 'number.ordinal',
                            '@default': 'number.float',
                        },
                    },
                ],
                [/(\.\.|\?:|[-+=/~?:;,.!@$%^&|*<>])/, 'delimiter'],
                [REG_ORDINAL, 'number.ordinal'],
            ],
            whitespace: [
                [/(@whitespace)+/, ''],
                [/\/\/.*$/, 'comment.line'],
                [/\/\*{2}/, 'comment.doc', '@doc_comment'],
                [/\/\*/, 'comment.block', '@block_comment'],
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
                Array.from({ length: MAX_VERBATIM_LENGTH }, (_, i) => {
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
                                String.raw`(${dollarRegex}\{)`,
                                {
                                    token: 'punctuation.section.embedded',
                                    bracket: '@open',
                                    next: '@braced',
                                },
                            ],
                            [
                                String.raw`(${dollarRegex}\()`,
                                {
                                    token: 'punctuation.section.embedded',
                                    bracket: '@open',
                                    next: '@parenthesized',
                                },
                            ],
                            [`\\\${0,${dollarCount}}`, 'string', '@pop'],
                            ['', '', '@pop'],
                        ],
                    ];
                }),
            ),
            string_interpolation: [[/\$*/, 'string', '@pop']],

            braced: [
                [/\{/, { token: '@brackets', next: '@braced_inner' }],
                [/\}/, { token: 'punctuation.section.embedded', bracket: '@close', next: '@pop' }],
                [/[[\]()]/, '@brackets'],
                { include: '@common' },
            ],
            braced_inner: [
                [/\{/, { token: '@brackets', next: '@push' }],
                [/\}/, { token: '@brackets', next: '@pop' }],
                [/[[\]()]/, '@brackets'],
                { include: '@common' },
            ],
            parenthesized: [
                [/\(/, { token: '@brackets', next: '@parenthesized_inner' }],
                [/\)/, { token: 'punctuation.section.embedded', bracket: '@close', next: '@pop' }],
                [/[[\]{}]/, '@brackets'],
                { include: '@common' },
            ],
            parenthesized_inner: [
                [/\(/, { token: '@brackets', next: '@push' }],
                [/\)/, { token: '@brackets', next: '@pop' }],
                [/[[\]{}]/, '@brackets'],
                { include: '@common' },
            ],

            block_comment: [
                [/\*\//, { token: 'comment.block', next: '@pop' }],
                [/[^*]+/, { token: 'comment.block' }],
                [/\*/, { token: 'comment.block' }],
            ],

            doc_comment: [
                [/\*\//, { token: 'comment.doc', next: '@pop' }],
                [/^(\s*)\*(?!\/)/, { token: 'comment.doc' }],
                [/\\\*(?!\/)/, { token: 'comment.doc.escape' }],
                [/@(param|returns)/, { token: 'entity.name.tag.doc' }],
                [/\*{2}(\S|\S.*?\S)\*{2}(?!\/)/, { token: 'comment.strong' }],
                [/\*(\S|\S.*?\S)\*(?!\/)/, { token: 'comment.emphasis' }],
                [/[^*@\\]+/, { token: 'comment.doc' }],
                [/[*@\\]/, { token: 'comment.doc' }],
            ],

            root_doc: [
                // inline doc, start with `\0`
                [/^\0/, { token: '', switchTo: '@inline_doc' }],
                [/(?=.)/, { token: '', switchTo: '@doc_mode' }],
            ],

            inline_doc: [
                [
                    /(@inlineDocParam)(@whitespace+)(\.\.|)(mut)(@whitespace+)(@identifier)/,
                    ['entity.name.label', '', 'delimiter', 'keyword.mut', '', 'variable.emphasis'],
                ],
                [
                    /(@inlineDocParam)(@whitespace+)(\.\.|)(@identifier)/,
                    ['entity.name.label', '', 'delimiter', 'variable.other.constant.emphasis'],
                ],
                [/(@whitespace*)(\(module\))(@whitespace*)(@identifier)/, ['', 'entity.name.label', '', 'type']],
                [/(\(@inlineDocMod\))(@whitespace+)/, ['entity.name.label', '']],
                { include: '@doc_mode' },
            ],

            doc_mode: [
                [
                    /(@identifier)(@whitespace*)(:)(@whitespace*)(\/\*@whitespace*<)(extern )((?:async )?function\*?)(>@whitespace*\*\/)/,
                    [
                        'entity.name.function.doc',
                        '',
                        'delimiter',
                        '',
                        'comment.doc',
                        'type.doc',
                        'keyword.javascript',
                        'comment.doc',
                    ],
                ],
                [
                    /(@identifier)(@whitespace*)(:)(@whitespace*)(\/\*@whitespace*<)(extern )(class)(@whitespace*)([<>.\w]*)(>@whitespace*\*\/)/,
                    [
                        'type.doc',
                        '',
                        'delimiter',
                        '',
                        'comment.doc',
                        'type.doc',
                        'keyword.javascript',
                        '',
                        'type.javascript',
                        'comment.doc',
                    ],
                ],
                [
                    /(@identifier)(@whitespace*)(:)(@whitespace*)(\/\*@whitespace*<)(extern )([\w]*)(>@whitespace*\*\/)/,
                    [
                        'variable.other.property.doc',
                        '',
                        'delimiter',
                        '',
                        'comment.doc',
                        'type.doc',
                        'type.javascript',
                        'comment.doc',
                    ],
                ],
                [
                    /(@identifier)(@whitespace*)(:)(@whitespace*)(\/\*@whitespace*<)(function )([.\w]*)(>@whitespace*\*\/)/,
                    [
                        'entity.name.function.doc',
                        '',
                        'delimiter',
                        '',
                        'comment.doc',
                        'type.doc',
                        'entity.name.label',
                        'comment.doc',
                    ],
                ],
                [/(@identifier)(@whitespace*)(:)(@whitespace+)/, ['variable.other.property', '', 'delimiter', '']],

                [
                    /(\/\*@whitespace*<)(extern )((?:async )?function\*?)(>@whitespace*\*\/)/,
                    ['comment.doc', 'type.doc', 'keyword.javascript', 'comment.doc'],
                ],
                [
                    /(\/\*@whitespace*<)(extern )(class)(@whitespace*)([<>.\w]*)(>@whitespace*\*\/)/,
                    ['comment.doc', 'type.doc', 'keyword.javascript', '', 'type.javascript', 'comment.doc'],
                ],
                [
                    /(\/\*@whitespace*<)(extern )([\w]*)(\()(\d+)(\))(>@whitespace*\*\/)/,
                    [
                        'comment.doc',
                        'type.doc',
                        'type.javascript',
                        'delimiter',
                        'number.doc',
                        'delimiter',
                        'comment.doc',
                    ],
                ],
                [
                    /(\/\*@whitespace*<)(extern )([\w]*)([^>]*)(>@whitespace*\*\/)/,
                    ['comment.doc', 'type.doc', 'type.javascript', '', 'comment.doc'],
                ],
                [
                    /(\/\*@whitespace*<)(function )([.\w]*)(>@whitespace*\*\/)/,
                    ['comment.doc', 'type.doc', 'entity.name.label', 'comment.doc'],
                ],
                [
                    /(\/\*@whitespace*<)(\w+@whitespace*)([.\w]*)(>@whitespace*\*\/)/,
                    ['comment.doc', 'type.doc', 'entity.name.label', 'comment.doc'],
                ],

                [/(let|const)(@whitespace+)(@identifier)/, [{ token: 'keyword.$1' }, '', 'variable.other.constant']],
                [/(fn)(@whitespace+)(@identifier)$/, ['keyword.fn.doc', '', 'entity.name.function.doc']],
                [
                    /(fn)(@whitespace+)(@identifier)(\()(\.\.)(\))/,
                    ['keyword.fn.doc', '', 'entity.name.function.doc', '@brackets', 'delimiter', '@brackets'],
                ],
                [
                    /(fn)(@whitespace+)(@identifier)/,
                    ['keyword.fn.doc', '', { token: 'entity.name.function.doc', next: '@type_doc' }],
                ],
                [
                    /(let)(@whitespace+)(mut)(@whitespace+)(@identifier)/,
                    [{ token: 'keyword.$1' }, '', 'keyword.mut', '', 'variable'],
                ],
                [/[[\](){}]/, '@brackets'],
                { include: '@common' },
            ],
            type_doc: [
                [/;/, { token: 'delimiter', next: '@pop', goBack: 1 }],
                [/(\()(\.\.|)(@identifier)(?=,|\))/, ['@brackets', 'delimiter', 'variable.emphasis']],
                [/(,)(@whitespace*)(\.\.|)(@identifier)(?=,|\))/, ['delimiter', '', 'delimiter', 'variable.emphasis']],
                [/(fn)(\()/, ['type', '@brackets']],
                [/(type)(\()(@identifier)(\))/, ['type', '@brackets', 'variable.emphasis.doc', '@brackets']],
                [
                    /(@identifier)(:)(@whitespace*)(fn)(\()/,
                    ['entity.name.function.emphasis.doc', 'delimiter', '', 'type', '@brackets'],
                ],
                [/(@identifier)(:)/, ['variable.emphasis', 'delimiter']],
                [/@identifier/, 'type'],
                [/[&|.,]/, 'delimiter'],
                [/->/, 'delimiter'],
                [/[[\]()]/, '@brackets'],
                { include: '@whitespace' },
            ],
        },
    };
}

/** 注册 Mirascript 的 TokensProvider */
export function registerMiraScriptTokensProvider(): IDisposable[] {
    return [
        languages.setMonarchTokensProvider('mirascript', getTokensProvider('script')),
        languages.setMonarchTokensProvider('mirascript-template', getTokensProvider('template')),
        languages.setMonarchTokensProvider('mirascript-doc', getTokensProvider('doc')),
    ];
}
