#[macro_export]
macro_rules! export_resource {
    ($resource:ident) => {
        struct $resource;

        impl $crate::bindings::exports::giong::provider::lifecycle::Guest for $resource {
            fn read(
                config: String,
            ) -> ::std::result::Result<$crate::bindings::enums::ChangeStatus, String>
            {
                $crate::block_on(crate::read(&config)).map_err(|error| error.to_string())
            }

            fn apply(config: String) -> ::std::result::Result<String, String> {
                $crate::block_on(crate::apply(&config)).map_err(|error| error.to_string())
            }

            fn delete(state: String) -> ::std::result::Result<(), String> {
                $crate::block_on(crate::delete(&state)).map_err(|error| error.to_string())
            }
        }

        $crate::bindings::export!($resource with_types_in $crate::bindings);
    };
}
