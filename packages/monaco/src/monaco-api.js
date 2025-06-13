let monaco = null;
export default monaco;

export let CancellationTokenSource,
    Emitter,
    KeyCode,
    KeyMod,
    MarkerSeverity,
    MarkerTag,
    Position,
    Range,
    Selection,
    SelectionDirection,
    Token,
    Uri,
    editor,
    languages,
    service,
    utils;

/**
 * 注册 MiraScript Monaco API，不实际启用任何功能。
 * @param {object} monacoApi - Monaco API 对象，必须包含特定方法和属性。
 */
export function registerMonacoApi(monacoApi) {
    if (monaco) {
        throw new Error('MiraScriptMonacoLoader has already been registered.');
    }
    if (
        !monacoApi ||
        typeof monacoApi !== 'object' ||
        // !monaco.languages ||
        // 'function' != typeof monaco.languages.register ||
        // 'function' != typeof monaco.languages.onLanguage ||
        // !monaco.editor ||
        // 'function' != typeof monaco.editor.create ||
        // 'function' != typeof monaco.editor.createModel ||
        // 'function' != typeof monaco.editor.createWebWorker ||
        'function' != typeof monacoApi.Uri ||
        'function' != typeof monacoApi.Range ||
        'function' != typeof monacoApi.Position ||
        'function' != typeof monacoApi.CancellationTokenSource ||
        'function' != typeof monacoApi.Emitter
    ) {
        throw new TypeError('Invalid Monaco editor instance provided.');
    }
    monaco = monacoApi;
    ({
        CancellationTokenSource,
        Emitter,
        KeyCode,
        KeyMod,
        MarkerSeverity,
        MarkerTag,
        Position,
        Range,
        Selection,
        SelectionDirection,
        Token,
        Uri,
        editor,
        languages,
        service,
        utils,
    } = monacoApi);
}
