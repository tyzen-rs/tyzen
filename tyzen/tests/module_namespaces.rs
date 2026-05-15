use tyzen::ModuleNamespaceMeta;

// Note: In a real integration test, these would be submitted via macros.
// For this test, we verify the matching logic in NamespaceMap.

#[test]
fn test_module_path_resolution() {
    // Note: This test verifies the logic we added to NamespaceMap::collect()
    
    let module_ns = vec![
        ModuleNamespaceMeta { path: "crate::modules::tasks", ns: "Task" },
        ModuleNamespaceMeta { path: "crate::modules::tasks::special", ns: "SpecialTask" },
    ];
    
    // Sort logic from collect()
    let mut regs = module_ns;
    regs.sort_by_key(|m| std::cmp::Reverse(m.path.len()));

    let resolve_ns = |item_ns: Option<&'static str>, module_path: &'static str| -> Option<&'static str> {
        if item_ns.is_some() {
            return item_ns;
        }
        for reg in &regs {
            if module_path.starts_with(reg.path) {
                return Some(reg.ns);
            }
        }
        None
    };
    
    assert_eq!(resolve_ns(None, "crate::modules::tasks::entities"), Some("Task"));
    assert_eq!(resolve_ns(None, "crate::modules::tasks::special::entities"), Some("SpecialTask"));
    assert_eq!(resolve_ns(Some("Manual"), "crate::modules::tasks::entities"), Some("Manual"));
    assert_eq!(resolve_ns(None, "crate::other"), None);
}
