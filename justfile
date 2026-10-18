set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

default:
    @just --list

check:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo test --workspace

fmt-check:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo fmt --all --check

build-release:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo build --release --workspace

stills:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo test -p yahboom-app -- --ignored write_page_stills

installer: build-release
    New-Item -ItemType Directory -Force -Path "{{justfile_directory()}}\\target\\release\\bundle" | Out-Null
    & "${env:ProgramFiles(x86)}\\NSIS\\makensis.exe" "{{justfile_directory()}}\\installer\\installer.nsi"
