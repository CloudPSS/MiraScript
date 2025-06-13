import type * as monaco from 'monaco-editor';

export * from 'monaco-editor';
export * as monaco from 'monaco-editor';

/** Monaco Api 属性 */
export type MonacoApi = {
    Range: typeof monaco.Range;
    Position: typeof monaco.Position;
    Emitter: typeof monaco.Emitter;
    CancellationTokenSource: typeof monaco.CancellationTokenSource;
    Uri: typeof monaco.Uri;
};

/**
 * 注册 MiraScript Monaco API，不实际启用任何功能。
 */
export declare function registerMonacoApi(monacoApi: MonacoApi): void;
