## Screenshots

<div align="center">
 
> Screenshots may be different from the final version

</div>

<div align="center">

<img width="1443" height="788" alt="image" src="https://github.com/user-attachments/assets/b235ede8-30b8-4edb-b2d9-a36cbaa32cc6" />
<br><br>
</div>
If you have a problem or a suggestion for a change, join us on Discord
https://discord.gg/ymm5sg664c

BETTER FN 

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

