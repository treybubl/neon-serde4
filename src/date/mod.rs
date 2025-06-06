mod de;
mod ser;

pub use ser::*;

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct DT<T>(T);

#[derive(Clone, Debug)]
pub struct JsDate;
