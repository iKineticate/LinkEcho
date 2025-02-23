
if (-not (Get-Process -Name explorer -ErrorAction SilentlyContinue)) {
    Start-Sleep -Seconds 2
    if (-not (Get-Process -Name explorer -ErrorAction SilentlyContinue)) {
        Start-Process explorer
    }
}
