/* eslint-disable no-console */
import { program } from '@commander-js/extra-typings';
import { lib } from '@mirascript/mirascript/subtle';
import { KEYWORDS, OPERATORS } from '@mirascript/help';
import { printLib } from './lib.js';
import { printTable } from './table.js';

const command = program.command('man');
command
    .description('显示 MiraScript 的手册')
    .argument(
        '<topic>',
        '要显示的主题，可以是 "keywords"、"operators"、"libraries" 或标准库中的函数/模块名（如 "matrix.add"）、关键字/操作符（如 "if"、"&&"）',
    )
    .action((topic) => {
        if (topic in KEYWORDS) {
            const desc = KEYWORDS[topic as keyof typeof KEYWORDS];
            console.log(desc);
        } else if (topic in OPERATORS) {
            const desc = OPERATORS[topic as keyof typeof OPERATORS];
            console.log(desc);
        } else if (printLib(topic)) {
            return;
        } else if (topic === 'keywords') {
            printTable(KEYWORDS, 'MiraScript 关键字：');
        } else if (topic === 'operators') {
            printTable(OPERATORS, 'MiraScript 操作符：');
        } else if (topic === 'libraries') {
            printTable(lib, 'MiraScript 标准库：', (v, k) => {
                if ('summary' in v) {
                    return v.summary ?? k;
                }
                return `模块 ${k}`;
            });
        } else {
            command.help({ error: true });
        }
    });
