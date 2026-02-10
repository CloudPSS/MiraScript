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
    if (monaco && monaco !== monacoApi) {
        throw new Error('MiraScriptMonacoLoader has already been registered.');
    }
    if (
        !monacoApi ||
        typeof monacoApi !== 'object' ||
        !monacoApi.languages ||
        // 'function' != typeof monacoApi.languages.register ||
        // 'function' != typeof monacoApi.languages.onLanguage ||
        !monacoApi.editor ||
        // 'function' != typeof monacoApi.editor.create ||
        // 'function' != typeof monacoApi.editor.createModel ||
        // 'function' != typeof monacoApi.editor.createWebWorker ||
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
