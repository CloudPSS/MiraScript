import { spawn } from 'node:child_process';
import path from 'node:path';

const packageDir = path.dirname(import.meta.dirname);
const cli = path.join(packageDir, 'cli.js');

interface RunResult {
    readonly code: number | null;
    readonly stdout: string;
    readonly stderr: string;
}

/** 在独立进程中运行 CLI，以覆盖真实参数解析、标准流和退出码。 */
export function run(args: readonly string[], input = ''): Promise<RunResult> {
    return new Promise((resolve, reject) => {
        const child = spawn(process.execPath, [cli, ...args], { cwd: packageDir });
        let stdout = '';
        let stderr = '';
        child.stdout.setEncoding('utf8').on('data', (chunk: string) => (stdout += chunk));
        child.stderr.setEncoding('utf8').on('data', (chunk: string) => (stderr += chunk));
        child.on('error', reject);
        child.on('close', (code) => resolve({ code, stdout, stderr }));
        child.stdin.end(input);
    });
}
