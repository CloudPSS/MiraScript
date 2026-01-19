import type { ScriptInput, TranspileOptions } from './types.js';
import { emit } from './emit/index.js';
import { generateBytecode } from './generate-bytecode.js';
import { DiagnosticCode, parseDiagnostics } from './diagnostic.js';
import { expose, WorkerResult } from '@cloudpss/worker/pool';

export default expose(async () => {
    const compiler = {
        /**
         * 生成 MiraScript 对应的 JavaScript 代码
         */
        async compile(script: ScriptInput, options: TranspileOptions) {
            const [bytecode, errors] = await generateBytecode(script, options);
            if (bytecode == null) {
                return WorkerResult([undefined, errors], [errors.buffer]);
            }
            const sourcemaps = options.sourceMap
                ? parseDiagnostics(script, errors, (c) => c === DiagnosticCode.SourceMap).sourcemaps
                : [];
            const generatedCode = emit(script, bytecode, sourcemaps, options);
            return WorkerResult([generatedCode, errors], [errors.buffer]);
        },
    };
    // 预热编译器
    await compiler.compile('{}', {});
    return compiler;
});
