use tyzen::Type;

#[derive(Type)]
pub enum CommonMeta {
    #[tyzen(code = "404", msg = "Not Found")]
    NotFound,
    #[tyzen(code = "500", msg = "Internal Error")]
    Internal,
}

#[derive(Type)]
#[tyzen(apply = CommonMeta)]
pub enum TaskError {
    NotFound,
    #[tyzen(msg = "Task busy")]
    Internal,
    Unknown,
}

#[test]
fn test_metadata_inheritance() {
    let mut found = false;
    for meta in inventory::iter::<tyzen::TypeMeta>() {
        if meta.name == "TaskError" {
            found = true;
            if let tyzen::meta::TypeStructure::Enum(e) = (meta.structure)() {
                let not_found = e.variants.iter().find(|v| v.name == "NotFound").expect("NotFound variant missing");
                assert!(not_found.attrs.iter().any(|(k, v)| *k == "code" && *v == tyzen::meta::AttrValue::Str("404")));
                assert!(not_found.attrs.iter().any(|(k, v)| *k == "msg" && *v == tyzen::meta::AttrValue::Str("Not Found")));

                let internal = e.variants.iter().find(|v| v.name == "Internal").expect("Internal variant missing");
                assert!(internal.attrs.iter().any(|(k, v)| *k == "code" && *v == tyzen::meta::AttrValue::Str("500")));
                assert!(internal.attrs.iter().any(|(k, v)| *k == "msg" && *v == tyzen::meta::AttrValue::Str("Task busy")));

                let unknown = e.variants.iter().find(|v| v.name == "Unknown").expect("Unknown variant missing");
                assert!(unknown.attrs.is_empty(), "Unknown variant should have no attrs, found: {:?}", unknown.attrs);
            } else {
                panic!("Expected Enum structure");
            }
        }
    }
    assert!(found, "TaskError metadata not found in inventory");
}
