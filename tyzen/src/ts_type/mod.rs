pub mod arrays;
pub mod collections;
pub mod external;
pub mod option;
pub mod primitives;
pub mod result;
pub mod tuples;
pub mod wrappers;

use crate::TsType;

pub(super) fn array_type<T: TsType + ?Sized>() -> String {
    let inner = T::ts_name();
    if inner.contains(" | ") {
        format!("({inner})[]")
    } else {
        format!("{inner}[]")
    }
}
