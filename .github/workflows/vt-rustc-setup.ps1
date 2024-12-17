$ErrorActionPreference = "Stop"

rustup toolchain install nightly-2024-10-21 --component rustc --component rust-std --component cargo
if ($LastExitCode -ne 0) {
    throw "failed to install nightly toolchain"
}

# Download the rustc compiler
Write-Host "Downloading Valthrun rustc"
Invoke-WebRequest $env:VT_RUSTC -OutFile "vt_rustc.zip" -Headers @{
    "Accept"               = "application/octet-stream"
    "Authorization"        = "Bearer $env:VT_RUSTC_AUTHORIZATION"
    "X-GitHub-Api-Version" = "2022-11-28"
}

Write-Host "Extracting Valthrun rustc"
Expand-Archive -Path "vt_rustc.zip" -DestinationPath "$(Get-Location)/vt_rustc" -Force

Write-Host "Linking Valthrun rustc"
rustup toolchain link vt-rust "$(Get-Location)/vt_rustc"
if ($LastExitCode -ne 0) {
    throw "failed to link Valthrun rust version"
}

rustup default vt-rust
if ($LastExitCode -ne 0) {
    throw "failed to default to the Valthrun rust version"
}

rustc -V -v
cargo -V -v