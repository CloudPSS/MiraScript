/* eslint-disable no-console */
import { readFile, stat } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { InvalidArgumentError, program } from '@commander-js/extra-typings';
import { execute } from './execute.js';
import { isNaN, PositiveInfinity } from '../helpers/utils.js';
import pkg from '#package.json' with { type: 'json' };
import { compileSync } from '../compiler/index.js';
import { configCheckpoint, type VmValue } from '../vm/index.js';

const DEFAULT_TIMEOUT = 3000;

export default program
    .name(pkg.name.split('/').pop()!)
    .version(pkg.version)
    .description(pkg.description)
    .option(
        '-v, --variable <key=value>',
        '设置全局变量，可以多次使用',
        (v, p) => {
            p = { ...p };
            const i = v.indexOf('=');
            if (i < 0) {
                p[v] = true;
            } else {
                const key = v.slice(0, i).trim();
                const value = v.slice(i + 1).trim();
                try {
                    const pv = compileSync(`return (${value});`)();
                    p[key] = pv ?? value;
                } catch {
                    p[key] = value;
                }
            }
            return p;
        },
        {} as Record<string, VmValue>,
    )
    .option('-t, --template', '使用模板模式')
    .option(
        '--timeout <ms>',
        '脚本执行超时时间（毫秒，0 表示不超时）',
        (v) => {
            const ms = Number.parseFloat(v);
            if (isNaN(ms) || ms < 0) {
                throw new InvalidArgumentError('超时时间必须是非负整数');
            }
            return ms;
        },
        DEFAULT_TIMEOUT,
    )
    .option('--no-template', '使用脚本模式')
    .option('-e, --eval <script>', '要执行的脚本')
    .argument('[script]', '要执行的脚本文件路径（如果提供了 -e 则忽略此参数）')
    .action(async (script, opt) => {
        configCheckpoint(opt.timeout || PositiveInfinity);
        if (opt.eval != null) {
            const template = !!opt.template;
            await execute(opt.eval, template, opt.variable, template ? 'eval.miratpl' : 'eval.mira');
            return;
        }
        if (script) {
            try {
                const s = await stat(script);
                if (!s.isFile()) {
                    console.error(`脚本路径不是文件: ${script}`);
                    process.exitCode = 2;
                    return;
                }
            } catch (ex) {
                if ((ex as NodeJS.ErrnoException).code === 'ENOENT') {
                    console.error(`脚本文件不存在: ${script}`);
                    process.exitCode = 2;
                } else if ((ex as NodeJS.ErrnoException).code === 'EACCES') {
                    console.error(`权限不足: ${(ex as NodeJS.ErrnoException).message}`);
                    process.exitCode = 3;
                } else {
                    console.error(`无法访问脚本文件: ${(ex as NodeJS.ErrnoException).message}`);
                    process.exitCode = 1;
                }
                return;
            }
            const context = await readFile(script, 'utf8');
            const template = opt.template ?? script.endsWith('.miratpl');
            await execute(context, template, opt.variable, pathToFileURL(script).href);
            return;
        }
        program.help({ error: true });
    });
