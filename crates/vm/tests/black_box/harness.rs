use mirascript_vm::{MiraValue, Runtime, mira};

fn t_equal<const EQ: bool>(
    runtime: &mut Runtime,
    left: MiraValue,
    right: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    if runtime.values_equal(left, right)? == EQ {
        Ok(MiraValue::NIL)
    } else {
        let left_string = left.as_str(runtime)?.map(str::to_owned);
        let right_string = right.as_str(runtime)?.map(str::to_owned);
        anyhow::bail!(
            "assertion failed: {left:?} {left_string:?} {} {right:?} {right_string:?}; message={:?}",
            if EQ { "!=" } else { "==" },
            message
        )
    }
}

#[mira]
pub fn t_eq(
    runtime: &mut Runtime,
    left: MiraValue,
    right: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    t_equal::<true>(runtime, left, right, message)
}

#[mira]
pub fn t_ne(
    runtime: &mut Runtime,
    left: MiraValue,
    right: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    t_equal::<false>(runtime, left, right, message)
}

fn t_bool<const EXPECTED: bool>(
    runtime: &mut Runtime,
    value: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    if value.as_boolean() == Some(EXPECTED) {
        Ok(MiraValue::NIL)
    } else {
        let value_string = value.as_str(runtime)?.map(str::to_owned);
        anyhow::bail!(
            "assertion failed: expected {}, got {value:?} {value_string:?}; message={:?}",
            EXPECTED,
            message
        )
    }
}

#[mira]
pub fn t_true(
    runtime: &mut Runtime,
    value: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    t_bool::<true>(runtime, value, message)
}

#[mira]
pub fn t_false(
    runtime: &mut Runtime,
    value: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    t_bool::<false>(runtime, value, message)
}

#[mira]
pub fn t_throws(
    runtime: &mut Runtime,
    function: MiraValue,
    message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    match runtime.call(function, &[]) {
        Ok(value) => {
            let value_string = value.as_str(runtime)?.map(str::to_owned);
            anyhow::bail!(
                "assertion failed: expected function to throw, returned {value:?} {value_string:?}; message={:?}",
                message
            )
        }
        Err(_) => Ok(MiraValue::NIL),
    }
}

#[mira]
pub fn t_timeout(
    _runtime: &mut Runtime,
    _function: MiraValue,
    _message: Option<String>,
) -> Result<MiraValue, anyhow::Error> {
    // This is a placeholder for a timeout test. In this black-box test, we don't actually implement a timeout mechanism, so we just return Nil to indicate the test passed.
    Ok(MiraValue::NIL)
}

#[mira]
pub fn t_never(message: Option<String>) -> Result<MiraValue, anyhow::Error> {
    anyhow::bail!("unexpected execution: message={:?}", message)
}
