# j1939scanner

This tool is a simple J1939 scan tool.  J1939DA is not embedded.  You should purchase a copy of J1939DA_MAR2021.xlsx from SAE.

## Building
j1939scanner uses RP1210 as the CAN adapter.  RP1210 requires 32 bit.

1. Use rust toolchaing stable-i686-pc-windows-gnu.

2. install the msys2 32 bit GTK libraries:
From: https://github.com/gtk-rs/gtk/issues/494

Steps that seems work on empty Windows7 VM

Install rust with rustup‑init.exe (as script in console install to "C:\users\USER" anyway)
with i686-pc-windows-gnu/stable/yes
Install MSYS2 from http://www.msys2.org
I do use link different from http://gtk-rs.org/docs-src/requirements.html as sourceforge.net asked login
Added PATH="/c/Users/${USER}/.cargo/bin:${PATH}" to C:\MSYS32\home\USER\.bash_profile
In MSYS2 MSYS 32 bit console
```
pacman -S git
pacman -S mingw-w64-i686-gtk3
pacman -S mingw-w64-i686-toolchain
default parameters
```
In MSYS2 MinGW 32-bit console
#check that gtk installed property
`pkg-config --modversion gtk+-3.0`

```
git clone https://github.com/gtk-rs/examples/
cd examples
cargo build
cargo run --bin basic
```
