use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};

use crate::{TsType, ts_type::array_type};

macro_rules! impl_array_like_ts_type {
    ($($t:ident),+ $(,)?) => {
        $(
            impl<T: TsType> TsType for $t<T> {
                fn ts_name() -> String {
                    array_type::<T>()
                }
            }
        )+
    };
}

impl_array_like_ts_type!(Vec, VecDeque, LinkedList, HashSet, BTreeSet);

impl<T: TsType> TsType for [T] {
    fn ts_name() -> String {
        array_type::<T>()
    }
}

fn safe_key(key: String) -> String {
    match key.as_str() {
        "number" | "string" | "boolean" => key,
        _ => "string".to_string(),
    }
}

impl<K: TsType, V: TsType> TsType for HashMap<K, V> {
    fn ts_name() -> String {
        format!("Record<{}, {}>", safe_key(K::ts_name()), V::ts_name())
    }
}

impl<K: TsType, V: TsType> TsType for BTreeMap<K, V> {
    fn ts_name() -> String {
        format!("Record<{}, {}>", safe_key(K::ts_name()), V::ts_name())
    }
}
