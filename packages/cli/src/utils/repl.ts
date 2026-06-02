import { start, Recoverable } from 'node:repl';
import { callbackify } from 'node:util';
import { once } from 'node:events';
import { isNativeError } from 'node:util/types';
import { compile, CompileError, createVmContext, type TranspileOptions, type VmAny } from '@mirascript/mirascript';
import { print } from './print.js';
import { DiagnosticCode, constants } from '@mirascript/mirascript/subtle';

/** 检查错误 */
function throwRecoverableError(error: unknown): never {
    if (!(error instanceof CompileError)) throw error;
    if (
        error.message.includes(DiagnosticCode[DiagnosticCode.MissingCloseBrace]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.MissingCloseBracket]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.MissingCloseParen]) ||
        error.message.includes(DiagnosticCode[DiagnosticCode.UnterminatedInterpolation]) ||
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
/** 启动 REPL */
export async function startRepl(): Promise<void> {
    const context: Record<string, VmAny> = Object.create(null);
    const vmContext = createVmContext(context);
    const evaluator = async (cmd: string, fileName: string): Promise<VmAny> => {
        const opt: TranspileOptions = { input_mode: 'Script', fileName, pretty: true, sourceMap: true };
        try {
            const code = cmd.replaceAll(/[\r\n]/g, '\n');
            const bind = REG_BINDING.exec(code);
            if (bind) {
                const varName = bind[1]!;
                const script = await compile(`${code};return ${varName};`, opt);
                const result = script(vmContext);
                context[varName] = result;
                return result;
            } else {
                const script = await compile(code, opt);
                return script(vmContext);
            }
        } catch (err) {
            throwRecoverableError(err);
        }
    };
    const repl = start({
        prompt: '> ',
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
    repl.on('reset', () => {
        for (const k of Object.keys(context)) {
            delete context[k];
        }
    });
    await once(repl, 'exit');
}
