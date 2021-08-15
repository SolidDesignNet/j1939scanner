# j1939scanner

This tool is a simple J1939 scan tool.  J1939DA is not embedded.  You should purchase a copy of J1939DA_MAR2021.xlsx from SAE.

**STATUS: pre-initial release.  I'm still getting the foundation down.**

Currently, the tool menu will allow you to select an adapter (once!), collects packets onto a broadcast channel, logs them from another thread, loads the J1939DA and displays a table of the J1939DA with searching and sorting.
![image](https://user-images.githubusercontent.com/1972001/129487044-159f8f2b-79af-4337-9b4a-9dfad60e05fb.png)
![image](https://user-images.githubusercontent.com/1972001/129487134-9f7a4a98-f893-4480-83f9-12e75a2d72f3.png)


Upcoming features are a broadcast signal logger, a specific signal logger (on request and filtered broadcast), and a fault table.

## Building
j1939scanner uses RP1210 as the CAN adapter.  RP1210 requires 32 bit.

1. Use rust toolchaing stable-i686-pc-windows-gnu.

2. install the msys2 32 bit GTK libraries:
From: https://github.com/gtk-rs/gtk/issues/494

Install rust with rustup‑init.exe (as script in console install to "C:\users\USER" anyway)
with i686-pc-windows-gnu/stable/yes
Install MSYS2 from http://www.msys2.org
I do use link different from http://gtk-rs.org/docs-src/requirements.html as sourceforge.net asked login
Added PATH="/c/Users/${USER}/.cargo/bin:${PATH}" to C:\MSYS32\home\USER\.bash_profile
In MSYS2 MSYS 32 bit console
```
pacman -S git
pacman -S mingw-i686-gtk3  // double check "i686" for 32 bit
pacman -S mingw-i686-toolchain
default parameters  // WHAT IS THIS?!
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
