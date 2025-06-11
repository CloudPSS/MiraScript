import type { Monaco, IDisposable } from '../index.js';
import { MAX_VERBATIM_LENGTH } from '../constants.js';
import type { languages } from '@private/monaco-editor';

const configuration = ({ languages }: Monaco): languages.LanguageConfiguration => ({
    comments: {
        lineComment: '//',
        blockComment: ['/*', '*/'],
    },
    brackets: [
        ['(', ')'],
        ['[', ']'],
        ['{', '}'],
    ],
    colorizedBracketPairs: [
        ['(', ')'],
        ['[', ']'],
        ['{', '}'],
        ...Array.from({ length: MAX_VERBATIM_LENGTH }).map(
            (_, i): languages.CharacterPair => ['$'.repeat(i) + '{', '}'],
        ),
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
                indentAction: languages.IndentAction.IndentOutdent,
                appendText: ' * ',
            },
        },
        {
            // e.g. /** ...|
            beforeText: /^\s*\/\*\*(?!\/)([^*]|\*(?!\/))*$/,
            action: {
                indentAction: languages.IndentAction.None,
                appendText: ' * ',
            },
        },
        {
            // e.g.  * ...|
            beforeText: /^(\t|( {2}))* \*( ([^*]|\*(?!\/))*)?$/,
            action: {
                indentAction: languages.IndentAction.None,
                appendText: '* ',
            },
        },
        {
            // e.g.  */|
            beforeText: /^(\t|( {2}))* \*\/\s*$/,
            action: {
                indentAction: languages.IndentAction.None,
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
export function setLanguageConfiguration(monaco: Monaco): IDisposable[] {
    return [
        monaco.languages.setLanguageConfiguration('mirascript', configuration(monaco)),
        monaco.languages.setLanguageConfiguration('mirascript-template', configuration(monaco)),
    ];
}
