use crate::TsType;

macro_rules! impl_tuple_ts_type {
    ($($name:ident),+ $(,)?) => {
        impl<$($name: TsType),+> TsType for ($($name,)+) {
            fn ts_name() -> String {
                let items = vec![$($name::ts_name()),+];
                format!("[{}]", items.join(", "))
            }
        }
    };
}

impl_tuple_ts_type!(A);
impl_tuple_ts_type!(A, B);
impl_tuple_ts_type!(A, B, C);
impl_tuple_ts_type!(A, B, C, D);
impl_tuple_ts_type!(A, B, C, D, E);
impl_tuple_ts_type!(A, B, C, D, E, F);
impl_tuple_ts_type!(A, B, C, D, E, F, G);
impl_tuple_ts_type!(A, B, C, D, E, F, G, H);
