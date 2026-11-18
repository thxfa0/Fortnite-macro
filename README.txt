BETTER FN 2.0 — Rust / Windows

REQUIREMENTS
- Windows 10 or 11, 64-bit
- Rust (rustup), stable MSVC toolchain
- Microsoft C++ Build Tools / Visual Studio Build Tools with the "Desktop development with C++" workload and the Windows SDK

INSTALLING RUST
1. Download rustup-init.exe (x64) from https://rust-lang.org/tools/install/
2. Run the installer and choose the default installation.
3. If the installer asks for the Visual Studio C++ Build Tools, install them.
4. Open a new PowerShell window and run: rustc --version and cargo --version

RUNNING
1. Extract the project folder.
2. Open PowerShell in the folder that contains Cargo.toml.
3. Run: cargo run

RELEASE BUILD (EXE)
After compiling, the release build will be located at:
target\release\better_fn.exe

To build the release version:
cargo build --release

Settings are saved in config.json next to the EXE file.

NOTE
The application uses the standard Windows Low-Level Keyboard Hook and SendInput.
It does not bypass or modify game protections. Windows or a game may reject
synthetic input; there is no guarantee it will work in every game.
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
