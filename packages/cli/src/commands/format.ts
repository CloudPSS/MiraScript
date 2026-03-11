/* eslint-disable no-console */
import fs from 'node:fs/promises';
import { program } from '@commander-js/extra-typings';
import { loadModule, type BcModule } from '@mirascript/bindings/wasm';

let templateConfig: BcModule.wasm.Config | null = null;
let scriptConfig: BcModule.wasm.Config | null = null;

/** 获取配置 */
async function getConfig(templateMode: boolean): Promise<BcModule.wasm.Config> {
    const mod = await loadModule();
    if (templateMode) {
        if (!templateConfig) {
            templateConfig = new mod.wasm.Config();
            templateConfig.input_mode = mod.wasm.InputMode.Template;
            templateConfig.trivia = true;
            templateConfig.diagnostic_position_encoding = mod.wasm.DiagnosticPositionEncoding.Utf8;
        }
        return templateConfig;
    } else {
        if (!scriptConfig) {
            scriptConfig = new mod.wasm.Config();
            scriptConfig.input_mode = mod.wasm.InputMode.Script;
            scriptConfig.trivia = true;
            scriptConfig.diagnostic_position_encoding = mod.wasm.DiagnosticPositionEncoding.Utf8;
        }
        return scriptConfig;
    }
}

/** 格式化 */
async function format(input: string, templateMode: boolean): Promise<string | undefined> {
    if (!input) return '';

    const mod = await loadModule();
    const compiler = new mod.wasm.MonacoCompiler(input, await getConfig(templateMode));
    try {
        if (!compiler.parse()) return undefined;
        const formatted = compiler.format();
        if (!formatted) return undefined;
        return formatted;
    } finally {
        compiler.free();
    }
}

const command = program.command('format');
command
    .description('格式化 MiraScript 脚本')
    .option('-w, --write', '直接修改文件')
    .option('-t, --template', '对无法推断类型的文件使用模板模式')
    .argument('<script...>', '要格式化的脚本文件路径或 glob，输入 "-" 表示从标准输入读取')
    .action(async (script, opt) => {
        if (script.length === 0) {
            command.help({ error: true });
        }
        if (script.length === 1 && script[0] === '-') {
            // 从标准输入读取
            let input = '';
            for await (const chunk of process.stdin) {
                input += chunk;
            }
            const output = await format(input, !!opt.template);
            if (output == null) {
                console.error('格式化失败');
                process.exit(1);
            }
            process.stdout.write(output);
            return;
        }
        for await (const file of fs.glob(script)) {
            const input = await fs.readFile(file, 'utf8');
            const output = await format(input, !!opt.template);
            if (opt.write) {
                if (output == null) {
                    console.error(`格式化失败: ${file}`);
                    continue;
                }
                if (output === input) {
                    console.warn(`文件未更改: ${file}`);
                    continue;
                }
                await fs.writeFile(file, output, 'utf8');
                console.warn(`已格式化文件: ${file}`);
            } else {
                process.stdout.write(`// File: ${file}\n`);
                process.stdout.write((output ?? input) + '\n');
            }
        }
    });
