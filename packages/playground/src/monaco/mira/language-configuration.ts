import { languages } from 'monaco-editor';
import { REG_IDENTIFIER, REG_HEX, REG_OCT, REG_BIN, REG_NUMBER } from './constants.js';

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
    wordPattern: new RegExp(
        `(${REG_IDENTIFIER.source}|${REG_HEX.source}|${REG_OCT.source}|${REG_BIN.source}|${REG_NUMBER.source}|\\d+)`,
        'gu',
    ),
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
    autoCloseBefore: ')}];,:. \n\t',
    folding: {
        markers: {
            start: /^\s*\/\/\s*#?region\b/gu,
            end: /^\s*\/\/\s*#?endregion\b/gu,
        },
    },
});
