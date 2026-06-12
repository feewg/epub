use std::process::Command;

#[test]
fn binary_runs_with_rust_log_env() {
    let bin = env!("CARGO_BIN_EXE_kaf-cli");
    let output = Command::new(bin)
        .env("RUST_LOG", "debug")
        .arg("--help")
        .output()
        .expect("应能启动二进制文件");

    assert!(
        output.status.success(),
        "带 RUST_LOG=debug 时 --help 应成功退出:\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("kaf-cli") || stdout.contains("Convert txt to epub ebook"),
        "帮助输出应包含程序名或简介"
    );
}
