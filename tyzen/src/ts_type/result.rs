use crate::TsType;

impl<T: TsType, E: TsType> TsType for Result<T, E> {
    fn ts_name() -> String {
        format!("Result<{}, {}>", T::ts_name(), E::ts_name())
    }
}
