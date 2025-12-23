use std::process::Command;

fn workspace_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_workspace-cli"))
}

#[test]
fn test_help() {
    let output = workspace_cli()
        .arg("--help")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("workspace-cli"));
    assert!(stdout.contains("gmail"));
    assert!(stdout.contains("drive"));
}

#[test]
fn test_gmail_help() {
    let output = workspace_cli()
        .args(["gmail", "--help"])
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("get"));
    assert!(stdout.contains("send"));
}

#[test]
fn test_drive_help() {
    let output = workspace_cli()
        .args(["drive", "--help"])
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}

#[test]
fn test_calendar_help() {
    let output = workspace_cli()
        .args(["calendar", "--help"])
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}

#[test]
fn test_auth_status() {
    let output = workspace_cli()
        .args(["auth", "status"])
        .output()
        .expect("Failed to execute");

    // Should succeed even when not authenticated
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("authenticated"));
}

#[test]
fn test_version() {
    let output = workspace_cli()
        .arg("--version")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_all_services_have_help() {
    let services = ["gmail", "drive", "calendar", "docs", "sheets", "slides", "tasks", "auth"];

    for service in services {
        let output = workspace_cli()
            .args([service, "--help"])
            .output()
            .expect(&format!("Failed to execute {} --help", service));

        assert!(output.status.success(), "Help failed for {}", service);
    }
}
