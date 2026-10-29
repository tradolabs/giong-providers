macro_rules! export_resource {
    ($resource:ident) => {
        wit_bindgen::generate!({ path: "wit", world: "provider" });

        struct $resource;

        impl exports::giong::provider::lifecycle::Guest for $resource {
            fn read(config: String) -> ::std::result::Result<Option<String>, String> {
                crate::read(&config).map_err(|error| error.to_string())
            }

            fn apply(
                config: String,
                current: Option<String>,
            ) -> ::std::result::Result<String, String> {
                crate::apply(&config, current.as_deref()).map_err(|error| error.to_string())
            }

            fn delete(state: String) -> ::std::result::Result<(), String> {
                crate::delete(&state).map_err(|error| error.to_string())
            }
        }

        export!($resource);
    };
}

pub(crate) use export_resource;
