#![allow(unknown_lints)]
#![deny(unused_variables)]
#![deny(unused_mut)]
#![deny(clippy)]
#![deny(clippy::pedantic)]
#![allow(stutter)]
#![recursion_limit = "128"]

//!
//! Neon-serde
//! ==========
//!
//! This crate is a utility to easily convert values between
//!
//! A `Handle<JsValue>` from the `neon` crate
//! and any value implementing `serde::{Serialize, Deserialize}`
//!
//! ## Usage
//!
//! #### `neon_serde::from_value`
//! Convert a `Handle<js::JsValue>` to
//! a type implementing `serde::Deserialize`
//!
//! #### `neon_serde::to_value`
//! Convert a value implementing `serde::Serialize` to
//! a `Handle<JsValue>`
//!
//!
//! ## Example
//!
//! ```rust,no_run
//! # #![allow(dead_code)]
//! use serde::{Serialize, Deserialize};
//! use neon_serde4 as neon_serde;
//! use neon_serde::export;
//!
//! use neon::prelude::*;
//!
//! #[derive(Serialize, Debug, Deserialize)]
//! struct AnObject {
//!     a: u32,
//!     b: Vec<f64>,
//!     c: String,
//! }
//!
//! export! {
//!     /// Say hello based on a persons name
//!     fn say_hello(name: String) -> String {
//!         format!("Hello, {}!", name)
//!     }
//!
//!     fn return_nothing(name: String) {
//!         println!("Hello, {name}!");
//!     }
//! }
//!
//! fn deserialize_something(mut cx: FunctionContext) -> JsResult<JsValue> {
//!     let arg0 = cx.argument::<JsValue>(0)?;
//!
//!     let arg0_value :AnObject = neon_serde::from_value(&mut cx, arg0)
//!         .or_else(|e| cx.throw_error(e.to_string()))
//!         .unwrap();
//!     println!("{:?}", arg0_value);
//!
//!     Ok(JsUndefined::new(&mut cx).upcast())
//! }
//!
//! fn serialize_something(mut cx: FunctionContext) -> JsResult<JsValue> {
//!     let value = AnObject {
//!         a: 1,
//!         b: vec![2f64, 3f64, 4f64],
//!         c: "a string".into()
//!     };
//!
//!     let js_value = neon_serde::to_value(&mut cx, &value)
//!         .or_else(|e| cx.throw_error(e.to_string()))
//!         .unwrap();
//!     Ok(js_value)
//! }
//!
//! ```
//!

pub mod de;
pub mod errors;
pub mod ser;

#[cfg(feature = "chrono")]
pub mod date;

mod macros;

pub use de::from_value;
pub use de::from_value_opt;
pub use ser::to_value;

#[cfg(test)]
mod tests {
    use super::*;
    use neon::prelude::*;

    #[test]
    fn test_it_compiles() {
        fn check<'j>(mut cx: FunctionContext<'j>) -> JsResult<'j, JsValue> {
            let result: () = {
                let arg: Handle<'j, JsValue> = cx.argument::<JsValue>(0)?;
                let () = from_value(&mut cx, arg)
                    .or_else(|e| cx.throw_error(e.to_string()))
                    .unwrap();
                ()
            };
            let result: Handle<'j, JsValue> = to_value(&mut cx, &result)
                .or_else(|e| cx.throw_error(e.to_string()))
                .unwrap();
            Ok(result)
        }

        let _ = check;
    }

    #[test]
    fn test_it_compiles_2() {
        fn check<'j>(mut cx: FunctionContext<'j>) -> JsResult<'j, JsValue> {
            let result: () = {
                let arg: Option<Handle<'j, JsValue>> = cx.argument_opt(0);
                let () = from_value_opt(&mut cx, arg)
                    .or_else(|e| cx.throw_error(e.to_string()))
                    .unwrap();
            };
            let result: Handle<'j, JsValue> = to_value(&mut cx, &result)
                .or_else(|e| cx.throw_error(e.to_string()))
                .unwrap();
            Ok(result)
        }

        let _ = check;
    }
}
