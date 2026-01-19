import { WorkerPool, type WorkerMethod } from '@cloudpss/worker/pool';
import type WorkerApi from './worker.js';

const pool = /*#__PURE__*/ new WorkerPool<typeof WorkerApi>(
    () => {
        return new Worker(new URL('#compiler/worker', import.meta.url), {
            type: 'module',
            name: '@mirascript/compiler',
        });
    },
    {
        name: '@mirascript/compiler',
    },
);

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compileWorker(
    ...args: Parameters<WorkerMethod<typeof WorkerApi, 'compile'>>
): Promise<ReturnType<WorkerMethod<typeof WorkerApi, 'compile'>>> {
    return await pool.call('compile', args);
}

/**
 * 清理工作池
 */
export function destroyWorkerPool(): void {
    pool.destroy();
}
