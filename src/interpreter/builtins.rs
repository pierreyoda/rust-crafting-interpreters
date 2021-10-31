use std::time::{SystemTime, UNIX_EPOCH};

use crate::{errors::Result, values::LoxValue};

pub fn build_lox_clock_builtin() -> LoxValue {
    LoxValue::NativeFunction {
        label: "clock".into(),
        arity: 0,
        execute: |_env, _arguments| -> Result<LoxValue> {
            let time_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            Ok(LoxValue::Number(time_since_epoch.as_secs_f64()))
        },
    }
}
