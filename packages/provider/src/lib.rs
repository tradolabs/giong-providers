/// Runs resource futures behind synchronous WIT lifecycle exports.
#[doc(hidden)]
pub use futures_executor::block_on;

mod macros;
pub mod utils;

pub mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "provider",
        pub_export_macro: true,
    });

    pub mod enums {
        pub use crate::bindings::exports::giong::provider::lifecycle::ChangeStatus;
    }

    pub mod helpers {
        use crate::bindings::enums;
        use crate::utils;
        use serde::Serialize;

        pub fn diff<S: Serialize, D: Serialize>(
            expected: &S,
            actual: &Option<D>,
        ) -> enums::ChangeStatus {
            if actual.is_none() {
                return enums::ChangeStatus::Created;
            }
            if !utils::matches(expected, actual.as_ref().unwrap()) {
                return enums::ChangeStatus::Updated;
            }
            return enums::ChangeStatus::Unchanged;
        }
    }
}
