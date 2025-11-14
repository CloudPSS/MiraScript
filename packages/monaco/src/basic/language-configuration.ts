import { type IDisposable, languages } from '../monaco-api.js';
import { MAX_VERBATIM_LENGTH } from '../constants.js';

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

export const configuration = (): languages.LanguageConfiguration => ({
    comments: {
        lineComment: { comment: '//' },
        blockComment: ['/*', '*/'],
    },
    brackets: [
        ...Array.from({ length: MAX_VERBATIM_LENGTH }).flatMap((_, i): languages.CharacterPair[] => {
            const prefix = '$'.repeat(MAX_VERBATIM_LENGTH - i - 1);
            return [
                [`${prefix}{`, '}'],
                [`${prefix}(`, ')'],
            ];
        }),
        ['[', ']'],
    ],
    wordPattern: /(-?\d+\.\w+([+-]\w*)?)|([^`~!#%^&*()\-=+[{\]}\\|;:'",.<>/?\s]+)/g,
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
        languages.setLanguageConfiguration('mirascript', configuration()),
        languages.setLanguageConfiguration('mirascript-template', configuration()),
        languages.setLanguageConfiguration('mirascript-doc', configuration()),
    ];
}
