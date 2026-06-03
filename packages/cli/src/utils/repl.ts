import { start, Recoverable } from 'node:repl';
import { homedir } from 'node:os';
import path from 'node:path';
import { callbackify } from 'node:util';
import { once } from 'node:events';
import { isNativeError } from 'node:util/types';
import {
    compile,
    CompileError,
    createVmContext,
    type TranspileOptions,
    type VmAny,
    type VmValue,
} from '@mirascript/mirascript';
import { DiagnosticCode, constants } from '@mirascript/mirascript/subtle';
import { print } from './print.js';
import { noColor } from './color.js';

/** 检查错误 */
function throwRecoverableError(error: unknown): never {
    if (!(error instanceof CompileError)) throw error;
    if (
        error.message.includes(DiagnosticCode[DiagnosticCode.MissingCloseBrace]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.MissingCloseBracket]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.MissingCloseParen]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.UnterminatedInterpolation]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.ExpressionExpected]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.PatternExpected]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.UnterminatedString]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.MissingSemicolon])
    ) {
        throw new Recoverable(error);
    }
    throw error;
}

const REG_BINDING = new RegExp(
    String.raw`^\s*(?:const|let|let\s+mut)\s+(${constants.REG_IDENTIFIER.source})\s*=\s*`,
    'u',
);
const REG_MODULE = new RegExp(String.raw`^\s*mod\s+(${constants.REG_IDENTIFIER.source})\s*\{`, 'u');

/** 启动 REPL */
export async function startRepl(): Promise<void> {
    let lastResult: VmValue = null;
    const context = new Map<string, VmValue>();
    const vmContext = createVmContext(
        (key) => {
            if (context.has(key)) {
                return context.get(key)!;
            }
            if (key === '$') {
                return lastResult;
            }
            return undefined;
        },
        () => {
            const keys = [...context.keys()];
            keys.push('$');
            return keys;
        },
    );
    const evaluator = async (cmd: string, fileName: string): Promise<VmAny> => {
        const opt: TranspileOptions = { input_mode: 'Script', fileName, pretty: true, sourceMap: true };
        try {
            const code = cmd.replaceAll(/[\r\n]/g, '\n');
            const bind = REG_BINDING.exec(code);
            if (bind) {
                const varName = bind[1]!;
                const script = await compile(`${code};return ${varName};`, opt);
                const result = script(vmContext);
                context.set(varName, result);
                return result;
            }
            const mod = REG_MODULE.exec(code);
            if (mod) {
                const modName = mod[1]!;
                const script = await compile(`${code};return ${modName};`, opt);
                const result = script(vmContext);
                context.set(modName, result);
                return result;
            }

            const script = await compile(code, opt);
            const result = script(vmContext);
            lastResult = result;
            return result;
        } catch (err) {
            throwRecoverableError(err);
        }
    };
    const repl = start({
        useColors: !noColor,
        useGlobal: false,
        eval: callbackify(async (cmd: string, _: unknown, fileName: string) => evaluator(cmd, fileName)),
        completer: (line: string) => {
            const lastToken = line.split(/[\s.]+/).pop() ?? '';
            return [vmContext.keys().filter((k) => k.startsWith(lastToken)), lastToken];
        },
        writer: (value: VmAny | Error) => {
            if (value === undefined) {
                return '';
            }
            if (isNativeError(value)) {
                return `${value.name}: ${value.message}`;
            }
            return print(value);
        },
    });
    repl.setupHistory({
        filePath: path.resolve(homedir(), '.mirascript_repl_history'),
        removeHistoryDuplicates: true,
        onHistoryFileLoaded: (err) => {
            // Ignore error if history file does not exist
        },
    });
    repl.on('reset', () => {
        context.clear();
        lastResult = null;
    });
    await once(repl, 'exit');
}
