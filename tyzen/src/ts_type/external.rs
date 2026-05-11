#[cfg(feature = "serde_json")]
impl crate::TsType for serde_json::Value {
    fn ts_name() -> String {
        "unknown".to_string()
    }
}

#[cfg(feature = "uuid")]
impl crate::TsType for uuid::Uuid {
    fn ts_name() -> String {
        "string".to_string()
    }
}

#[cfg(feature = "chrono")]
mod chrono_impls {
    use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

    impl crate::TsType for Utc {
        fn ts_name() -> String {
            "".to_string()
        }
    }

    impl crate::TsType for Local {
        fn ts_name() -> String {
            "".to_string()
        }
    }

    impl<Tz: TimeZone + crate::TsType> crate::TsType for DateTime<Tz> {
        fn ts_name() -> String {
            "string".to_string()
        }
    }

    impl crate::TsType for NaiveDateTime {
        fn ts_name() -> String {
            "string".to_string()
        }
    }

    impl crate::TsType for NaiveDate {
        fn ts_name() -> String {
            "string".to_string()
        }
    }

    impl crate::TsType for NaiveTime {
        fn ts_name() -> String {
            "string".to_string()
        }
    }
}
