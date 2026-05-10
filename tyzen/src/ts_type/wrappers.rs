use std::{borrow::Cow, rc::Rc, sync::Arc};

use crate::TsType;

impl<T: TsType + ?Sized> TsType for &T {
    fn ts_name() -> String {
        T::ts_name()
    }
}

impl<T: TsType + ?Sized> TsType for Box<T> {
    fn ts_name() -> String {
        T::ts_name()
    }
}

impl<T: TsType + ?Sized> TsType for Rc<T> {
    fn ts_name() -> String {
        T::ts_name()
    }
}

impl<T: TsType + ?Sized> TsType for Arc<T> {
    fn ts_name() -> String {
        T::ts_name()
    }
}

impl<T> TsType for Cow<'_, T>
where
    T: TsType + ToOwned + ?Sized,
{
    fn ts_name() -> String {
        T::ts_name()
    }
}
