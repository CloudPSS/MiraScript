use mira_core::{Config, compile::CompileResult};
use napi::{
    Either, Env, Result, Task,
    bindgen_prelude::{AsyncTask, Object, Uint8Array, Uint32Array},
};
use napi_derive::napi;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
    let config: Config = env.from_js_value(config)?;
    Ok(Compile { script, config })
}

fn compile_impl(args: &Compile) -> CompileResult {
    mira_core::Compiler::compile(&args.script, &args.config)
}

fn to_result(data: CompileResult) -> JsCompileResult {
    let (chunk, diagnostics) = data;
    JsCompileResult {
        chunk: chunk.map(Uint8Array::new),
        diagnostics: Uint32Array::new(diagnostics),
    }
}

#[napi]
pub fn compile_sync(
    env: Env,
    script: Either<String, Uint8Array>,
    config: Object,
) -> Result<JsCompileResult> {
    let args = extract_args(&env, script, config)?;
    let data = compile_impl(&args);
    Ok(to_result(data))
}

pub struct Compile {
    pub script: String,
    pub config: Config,
}

#[napi]
impl Task for Compile {
    type Output = CompileResult;
    type JsValue = JsCompileResult;

    fn compute(&mut self) -> Result<Self::Output> {
        Ok(compile_impl(self))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(to_result(output))
    }
}

#[napi]
pub fn compile(
    env: Env,
    script: Either<String, Uint8Array>,
    config: Object,
) -> Result<AsyncTask<Compile>> {
    let args = extract_args(&env, script, config)?;
    Ok(AsyncTask::new(args))
}
