use crate::{TsType, ts_type::array_type};

impl<T: TsType, const N: usize> TsType for [T; N] {
    fn ts_name() -> String {
        array_type::<T>()
    }
}

#[cfg(test)]
mod tests {
    use crate::TsType;

    #[test]
    fn fixed_array_i32() {
        assert_eq!(<[i32; 4]>::ts_name(), "number[]");
    }

    #[test]
    fn fixed_array_string() {
        assert_eq!(<[String; 3]>::ts_name(), "string[]");
    }

    #[test]
    fn fixed_array_bool() {
        assert_eq!(<[bool; 2]>::ts_name(), "boolean[]");
    }

    #[test]
    fn fixed_array_size_1() {
        assert_eq!(<[i32; 1]>::ts_name(), "number[]");
    }

    #[test]
    fn fixed_array_size_0() {
        assert_eq!(<[i32; 0]>::ts_name(), "number[]");
    }

    #[test]
    fn fixed_array_nested() {
        assert_eq!(<[[i32; 2]; 3]>::ts_name(), "number[][]");
    }
}
