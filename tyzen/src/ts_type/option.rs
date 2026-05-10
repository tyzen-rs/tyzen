use crate::TsType;

impl<T: TsType> TsType for Option<T> {
    fn ts_name() -> String {
        format!("{} | null", T::ts_name())
    }
}
