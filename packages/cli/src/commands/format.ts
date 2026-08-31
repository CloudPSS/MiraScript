/* eslint-disable no-console */
import fs from 'node:fs/promises';
import { text as readText } from 'node:stream/consumers';
import { InvalidArgumentError, program } from '@commander-js/extra-typings';
import { loadModule } from '@mirascript/bindings';
import { formatDiagnosticMessage, parseDiagnostics } from '@mirascript/mirascript/subtle';

const DEFAULT_LINE_WIDTH = 80;
const DEFAULT_TAB_SIZE = 2;

/** CLI 格式化参数。 */
type FormatterOptions = {
    readonly lineWidth: number;
    readonly tabSize: number;
    readonly useTabs: boolean;
};

/** 创建正整数参数解析器。 */
function positiveInteger(name: string): (value: string) => number {
    return (value) => {
        const parsed = Number(value);
        if (!Number.isSafeInteger(parsed) || parsed <= 0) {
            throw new InvalidArgumentError(`${name} 必须是正整数`);
        }
        return parsed;
    };
}

/** 按倒序应用 UTF-16 偏移编辑。 */
function applyEdits(source: string, edits: ReadonlyArray<{ start: number; end: number; text: string }>): string {
    let output = source;
    for (const edit of edits.toSorted((left, right) => right.start - left.start)) {
        output = output.slice(0, edit.start) + edit.text + output.slice(edit.end);
    }
    return output;
}

/** 使用统一 binding 格式化一份源码。 */
async function format(input: string, templateMode: boolean, options: FormatterOptions): Promise<string | undefined> {
    const module = await loadModule();
    const result = module.formatSync(
        input,
        {
            input_mode: templateMode ? 'Template' : 'Script',
            trivia: true,
            // MiraScript 配置枚举的公开序列化值固定为 Utf8。
            // eslint-disable-next-line unicorn/text-encoding-identifier-case
            diagnostic_position_encoding: 'Utf8',
        },
        {
            tabSize: options.tabSize,
            insertSpaces: !options.useTabs,
            printWidth: options.lineWidth,
        },
    );
    const parsed = parseDiagnostics(input, result.diagnostics);
    if (parsed.errors.length > 0) {
        for (const diagnostic of parsed.errors) {
            console.error(formatDiagnosticMessage(diagnostic.code));
        }
        return undefined;
    }
    return applyEdits(input, result.edits);
}

const command = program.command('format');
command
    .description('格式化 MiraScript 脚本')
    .option('-w, --write', '直接修改文件')
    .option('-c, --check', '检查文件是否已经格式化')
    .option('-t, --template', '使用模板模式；文件默认根据 .miratpl 扩展名推断')
    .option('--no-template', '强制使用脚本模式')
    .option('--line-width <columns>', '最大行宽', positiveInteger('最大行宽'), DEFAULT_LINE_WIDTH)
    .option('--tab-size <columns>', '缩进宽度', positiveInteger('缩进宽度'), DEFAULT_TAB_SIZE)
    .option('--use-tabs', '使用制表符缩进')
    .argument('<script...>', '要格式化的脚本文件路径或 glob，输入 "-" 表示从标准输入读取')
    .action(async (patterns, options) => {
        if (options.write && options.check) {
            console.error('--write 与 --check 不能同时使用');
            process.exitCode = 2;
            return;
        }
        const formatterOptions: FormatterOptions = {
            lineWidth: options.lineWidth,
            tabSize: options.tabSize,
            useTabs: !!options.useTabs,
        };
        if (patterns.length === 1 && patterns[0] === '-') {
            if (options.write || options.check) {
                console.error('标准输入不能与 --write 或 --check 一起使用');
                process.exitCode = 2;
                return;
            }
            const input = await readText(process.stdin);
            const output = await format(input, options.template === true, formatterOptions);
            if (output == null) {
                console.error('格式化失败');
                process.exitCode = 1;
                return;
            }
            process.stdout.write(output);
            return;
        }
        if (patterns.includes('-')) {
            console.error('标准输入不能与文件路径混用');
            process.exitCode = 2;
            return;
        }

        const files = new Set<string>();
        for await (const file of fs.glob(patterns)) files.add(file);
        const sortedFiles = [...files].toSorted();
        if (sortedFiles.length === 0) {
            console.error('没有匹配到任何脚本文件');
            process.exitCode = 2;
            return;
        }

        let failed = false;
        for (const [index, file] of sortedFiles.entries()) {
            try {
                const input = await fs.readFile(file, 'utf8');
                const template = options.template ?? file.toLowerCase().endsWith('.miratpl');
                const output = await format(input, template, formatterOptions);
                if (output == null) {
                    console.error(`格式化失败: ${file}`);
                    failed = true;
                    continue;
                }
                if (options.check) {
                    if (output !== input) {
                        console.error(`需要格式化: ${file}`);
                        failed = true;
                    }
                } else if (options.write) {
                    if (output !== input) {
                        await fs.writeFile(file, output, 'utf8');
                        console.error(`已格式化: ${file}`);
                    }
                } else if (sortedFiles.length === 1) {
                    process.stdout.write(output);
                } else {
                    if (index > 0) process.stdout.write('\n');
                    process.stdout.write(`// File: ${file}\n${output}`);
                }
            } catch (error) {
                console.error(`无法处理文件 ${file}: ${error instanceof Error ? error.message : String(error)}`);
                failed = true;
            }
        }
        if (failed) process.exitCode = 1;
    });
