import { languages } from '@private/monaco-editor';

languages.setLanguageConfiguration('mirascript', {
    comments: {
        lineComment: '//',
        blockComment: ['/*', '*/'],
    },
    brackets: [
        ['(', ')'],
        ['[', ']'],
        ['{', '}'],
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
