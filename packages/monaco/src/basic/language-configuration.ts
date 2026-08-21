import { type IDisposable, languages } from '../monaco-api.js';
import { MAX_VERBATIM_LENGTH } from '../constants.js';
import { CONTRIBUTE_IDS } from '../contribute.js';

/** 缩进配置 */
function indentAction(action: keyof typeof languages.IndentAction): { indentAction: languages.IndentAction } {
    if (languages == null) {
        // vscode
        return {
            indent: action[0]?.toLowerCase() + action.slice(1),
        } as unknown as { indentAction: languages.IndentAction };
    }
    return { indentAction: languages.IndentAction[action] };
}

/** 括号配置 */
function brackets(): languages.CharacterPair[] {
    const brackets: languages.CharacterPair[] = [];
    for (let i = 0; i < MAX_VERBATIM_LENGTH; i++) {
        const prefix = '$'.repeat(MAX_VERBATIM_LENGTH - i - 1);
        brackets.push([`${prefix}{`, '}'], [`${prefix}(`, ')']);
    }
    brackets.push(['[', ']']);
    return brackets;
}

export const configuration = (): languages.LanguageConfiguration => ({
    comments: {
        lineComment: { comment: '//' },
        blockComment: ['/*', '*/'],
    },
    brackets: brackets(),
    wordPattern: /((?<!\.\s*)[\d_]+\.[\d_]+([eE][+-]?[\d_]+)?)|([^`~!#%^&*()\-=+[{\]}\\|;:'",.<>/?\s]+)/g,
    autoClosingPairs: [
        { open: '{', close: '}' },
        { open: '[', close: ']' },
        { open: '(', close: ')' },
        { open: '"', close: '"', notIn: ['string'] },
        { open: "'", close: "'", notIn: ['string', 'comment'] },
        { open: '`', close: '`', notIn: ['string', 'comment'] },
        { open: '/**', close: ' */', notIn: ['string'] },
    ],
    onEnterRules: [
        {
            // e.g. /** | */
            beforeText: /^\s*\/\*\*(?!\/)([^*]|\*(?!\/))*$/,
            afterText: /^\s*\*\/$/,
            action: {
                ...indentAction('IndentOutdent'),
                appendText: ' * ',
            },
        },
        {
            // e.g. /** ...|
            beforeText: /^\s*\/\*\*(?!\/)([^*]|\*(?!\/))*$/,
            action: {
                ...indentAction('None'),
                appendText: ' * ',
            },
        },
        {
            // e.g.  * ...|
            beforeText: /^(\t|( {2}))* \*( ([^*]|\*(?!\/))*)?$/,
            action: {
                ...indentAction('None'),
                appendText: '* ',
            },
        },
        {
            // e.g.  */|
            beforeText: /^(\t|( {2}))* \*\/\s*$/,
            action: {
                ...indentAction('None'),
                removeText: 1,
            },
        },
    ],
    autoCloseBefore: '*/%^ &| ><= )}] ;,?: \'"` \n\t ~#\\',
    folding: {
        markers: {
            start: /^\s*\/\/#?\s*?region\b/gu,
            end: /^\s*\/\/#?\s*?endregion\b/gu,
        },
    },
});

/** 注册语言配置 */
export function setLanguageConfiguration(): IDisposable[] {
    return [
        languages.setLanguageConfiguration(CONTRIBUTE_IDS.mirascript, configuration()),
        languages.setLanguageConfiguration(CONTRIBUTE_IDS.mirascriptTemplate, configuration()),
        languages.setLanguageConfiguration(CONTRIBUTE_IDS.mirascriptDoc, configuration()),
    ];
}
