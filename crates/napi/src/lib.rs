use mira_core::Config;
use neon::{
    prelude::*,
    types::{
        buffer::TypedArray,
        extract::{Json, TryFromJs},
    },
};

struct JsCompileResult(Option<Vec<u8>>, Vec<u32>);

impl JsCompileResult {
    pub fn try_into_js<'cx>(self, cx: &mut Cx<'cx>) -> JsResult<'cx, JsObject> {
        Ok(cx
            .empty_object()
            .prop(cx, "chunk")
            .set(self.0)?
            .prop("diagnostics")
            .set(self.1)?
            .this())
    }
}

fn extract_args<'cx>(cx: &mut FunctionContext<'cx>) -> NeonResult<(String, Config)> {
    let script = cx.argument::<JsValue>(0)?;
    let script = script
        .downcast::<JsString, _>(cx)
        .map(|v| v.value(cx))
        .or_else(|_| {
            let buffer = script.downcast::<JsUint8Array, _>(cx).or_else(|_| {
                let err = cx.type_error("Expected string or buffer")?;
                cx.throw(err)?
            })?;
            let slice = buffer.as_slice(cx);
            Ok(String::from_utf8_lossy(slice).into_owned())
        })?;

    let config = cx.argument::<JsValue>(1)?;
    let config = Json::<Config>::from_js(cx, config)?;

    Ok((script, config.0))
}

fn compile_impl(script: String, config: Config) -> JsCompileResult {
    let (chunk, diagnostics) = mira_core::compile(&script, &config);
    JsCompileResult(chunk, diagnostics)
}

#[neon::export]
pub fn compile_sync<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsObject> {
    let (script, config) = extract_args(cx)?;
    let result = compile_impl(script, config);
    result.try_into_js(cx)
}

#[neon::export]
pub fn compile<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsPromise> {
    let (script, config) = extract_args(cx)?;
    Ok(cx
        .task(move || compile_impl(script, config))
        .promise(|mut cx, result| result.try_into_js(&mut cx)))
}
