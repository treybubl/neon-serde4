//!
//! Defines macros for easily exporting functions
//!

#[macro_export]
macro_rules! export {

    ($(
        $(#[$func_meta:meta])*
        fn $name:ident($( $arg:ident : $atype:ty ),*) $(-> $ret:ty)? $code:block
    )*) => (
        $(
            #[allow(non_snake_case)]
            $(#[$func_meta])*
            fn $name($( $arg: $atype ),*) $(-> $ret)? $code
        )*

        #[neon::main]
        pub fn main(mut m: ModuleContext) -> Result<(), neon::result::Throw> {
            $(
                m.export_function(stringify!($name), |mut cx| {
                    use neon::result::ResultExt;

                    // Can be done away with a fancier macro
                    let mut _arg_index = 0;

                    $(
                        let $arg = cx.argument_opt(_arg_index);
                        let $arg: $atype = $crate::from_value_opt(&mut cx, $arg)
                            .or_else(|err| err.or_throw(&mut cx))?;
                        _arg_index += 1;
                    )*

                    let result = $name($( $arg ),*);
                    let handle = $crate::export!( @handle cx, result, $($ret)? );
                    Ok(handle)
                })?;
            )*
            Ok(())
        }
    );

    ( @handle $cx:ident, $result:ident, $ret:ty ) => (
        $crate::to_value(&mut $cx, &$result).or_else(|err| err.or_throw(&mut $cx))?
    );

    ( @handle $cx:ident, $result:ident, ) => ( $cx.undefined() );
}
