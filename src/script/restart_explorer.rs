pub fn restart_explorer() {
    let _ = std::process::Command::new("taskkill")
        .args(["/f", "/im", "explorer.exe"])
        .output();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let _ = std::process::Command::new("explorer.exe").spawn();
}
