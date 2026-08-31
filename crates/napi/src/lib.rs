use std::panic::{AssertUnwindSafe, catch_unwind};

use mirascript_core::{CompileConfig, CompileResult};
use napi::{
    Either, Env, Result, Task,
    bindgen_prelude::{AsyncTask, Object, Uint8Array, Uint32Array},
};
use napi_derive::napi;

#[napi]
pub struct JsCompileResult {
    pub chunk: Option<Uint8Array>,
    pub diagnostics: Uint32Array,
}

fn extract_args(env: &Env, script: Either<String, Uint8Array>, config: Object) -> Result<Compile> {
    let script = match script {
        Either::A(s) => s,
        Either::B(arr) => {
            let slice = arr.as_ref();
            String::from_utf8_lossy(slice).into_owned()
        }
    };
    let config: CompileConfig = env.from_js_value(config)?;
    Ok(Compile { script, config })
}

fn compile_impl(args: &Compile) -> CompileResult {
    mirascript_core::Compiler::compile(&args.script, &args.config)
}

fn to_result(data: CompileResult) -> JsCompileResult {
    let (chunk, diagnostics) = data;
    JsCompileResult {
        chunk: chunk.map(Uint8Array::new),
        diagnostics: Uint32Array::new(diagnostics),
    }
}

fn wrap_panic<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    catch_unwind(AssertUnwindSafe(f))
        .map_err(|_| napi::Error::from_reason("Panic occurred".to_string()))?
}

#[napi]
pub fn compile_sync(
    env: Env,
    script: Either<String, Uint8Array>,
    config: Object,
) -> Result<JsCompileResult> {
    wrap_panic(|| {
        let args = extract_args(&env, script, config)?;
        let data = compile_impl(&args);
        Ok(to_result(data))
    })
}

pub struct Compile {
    pub script: String,
    pub config: CompileConfig,
}

#[napi]
impl Task for Compile {
    type Output = CompileResult;
    type JsValue = JsCompileResult;

    fn compute(&mut self) -> Result<Self::Output> {
        wrap_panic(|| Ok(compile_impl(self)))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        wrap_panic(|| Ok(to_result(output)))
    }
}

#[napi]
pub fn compile(
    env: Env,
    script: Either<String, Uint8Array>,
    config: Object,
) -> Result<AsyncTask<Compile>> {
    wrap_panic(|| {
        let args = extract_args(&env, script, config)?;
        Ok(AsyncTask::new(args))
    })
}

#[napi(object)]
pub struct FormatOptions {
    pub tab_size: Option<u32>,
    pub insert_spaces: Option<bool>,
    pub print_width: Option<u32>,
}

#[napi(object)]
pub struct FormatRange {
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
pub struct FormatEdit {
    pub start: u32,
    pub end: u32,
    pub text: String,
}

#[napi(object)]
pub struct FormatResult {
    pub diagnostics: Uint32Array,
    pub edits: Vec<FormatEdit>,
}

fn utf16_to_utf8(source: &str, offset: usize) -> Option<usize> {
    let mut utf16 = 0;
    for (utf8, character) in source.char_indices() {
        if utf16 == offset {
            return Some(utf8);
        }
        utf16 += character.len_utf16();
        if utf16 > offset {
            return None;
        }
    }
    (utf16 == offset).then_some(source.len())
}

fn utf8_to_utf16(source: &str, offset: usize) -> u32 {
    source[..offset].encode_utf16().count() as u32
}

#[napi]
pub fn format_sync(
    env: Env,
    source: String,
    config: Object,
    options: Option<FormatOptions>,
    ranges: Option<Vec<FormatRange>>,
) -> Result<FormatResult> {
    wrap_panic(|| {
        let config: CompileConfig = env.from_js_value(config)?;
        let options = options.unwrap_or(FormatOptions {
            tab_size: None,
            insert_spaces: None,
            print_width: None,
        });
        let defaults = mirascript_core::formatter::FormatOptions::default();
        let options = mirascript_core::formatter::FormatOptions {
            tab_size: options
                .tab_size
                .map_or(defaults.tab_size, |value| value as usize),
            use_spaces: options.insert_spaces.unwrap_or(defaults.use_spaces),
            line_width: options
                .print_width
                .map_or(defaults.line_width, |value| value as usize),
        };
        let ranges = ranges.unwrap_or_default();
        let (edits, diagnostics) = if ranges.is_empty() {
            let outcome = mirascript_core::formatter::format_document(&source, &config, &options)
                .map_err(|error| napi::Error::from_reason(error.to_string()))?;
            let edits = outcome
                .value
                .filter(|text| text != &source)
                .map(|text| mirascript_core::formatter::FormatEdit {
                    range: 0..source.len(),
                    text,
                })
                .into_iter()
                .collect();
            (edits, outcome.diagnostics)
        } else {
            let ranges = ranges
                .into_iter()
                .map(|range| {
                    let start = utf16_to_utf8(&source, range.start as usize).ok_or_else(|| {
                        napi::Error::from_reason("invalid UTF-16 format range start")
                    })?;
                    let end = utf16_to_utf8(&source, range.end as usize).ok_or_else(|| {
                        napi::Error::from_reason("invalid UTF-16 format range end")
                    })?;
                    Ok(start..end)
                })
                .collect::<Result<Vec<_>>>()?;
            let outcome =
                mirascript_core::formatter::format_ranges(&source, &config, &ranges, &options)
                    .map_err(|error| napi::Error::from_reason(error.to_string()))?;
            (outcome.value.unwrap_or_default(), outcome.diagnostics)
        };
        let diagnostics = mirascript_core::encode_diagnostics(&source, &diagnostics, &config);
        let edits = edits
            .into_iter()
            .map(|edit| FormatEdit {
                start: utf8_to_utf16(&source, edit.range.start),
                end: utf8_to_utf16(&source, edit.range.end),
                text: edit.text,
            })
            .collect();
        Ok(FormatResult {
            diagnostics: Uint32Array::new(diagnostics),
            edits,
        })
    })
}
