BETTER FN 2.0 — Rust / Windows

WYMAGANIA
- Windows 10 lub 11, 64-bit
- Rust (rustup), toolchain stable MSVC
- Microsoft C++ Build Tools / Visual Studio Build Tools z workloadem "Desktop development with C++" i Windows SDK

INSTALACJA RUST
1. Pobierz rustup-init.exe x64 z https://rust-lang.org/tools/install/
2. Uruchom instalator i wybierz domyślną instalację.
3. Jeśli instalator poprosi o Visual Studio C++ Build Tools, zainstaluj je.
4. Otwórz nowe okno PowerShell i wpisz: rustc --version oraz cargo --version

URUCHOMIENIE
1. Rozpakuj folder projektu.
2. Otwórz PowerShell w folderze zawierającym Cargo.toml.
3. Wpisz: cargo run

WERSJA GOTOWA (EXE)
Po kompilacji wydanie będzie w:
target\release\better_fn.exe

Aby zbudować wersję release:
cargo build --release

Ustawienia są zapisywane w config.json obok pliku EXE.

UWAGA
Aplikacja korzysta ze standardowego Windows Low-Level Keyboard Hook i SendInput.
Nie omija ani nie modyfikuje zabezpieczeń gier. Windows lub gra mogą odrzucać
syntetyczne wejście; Rust nie gwarantuje działania w każdej grze.
