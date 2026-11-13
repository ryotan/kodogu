use kodogu_runtime::panic::install_hook;
use std::{panic, process::Command};

const CHILD: &str = "KODOGU_RUNTIME_PANIC_TEST";
const PRIVATE: &str = "private panic payload must not reach stderr";

#[test]
fn a_hook_reports_only_location_and_preserves_the_caught_payload() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "subprocess_hook", "--nocapture"])
        .env(CHILD, "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let location: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
    assert_eq!(location["file"], file!());
    assert!(location["line"].as_u64().unwrap() > 0);
    assert!(location["column"].as_u64().unwrap() > 0);
    assert_eq!(location.as_object().unwrap().len(), 3);
    assert!(!stderr.contains(PRIVATE));
}

#[test]
fn subprocess_hook() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    install_hook(|location| {
        let location = location.unwrap();
        eprintln!(
            "{}",
            serde_json::json!({
                "file":location.file, "line":location.line, "column":location.column,
            })
        );
    });
    let error = panic::catch_unwind(|| panic!("{PRIVATE}")).unwrap_err();
    assert_eq!(error.downcast_ref::<String>().unwrap(), PRIVATE);
    std::process::exit(0);
}
