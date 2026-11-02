<div align="center">
<h1>WoeUSB (Rust)</h1>
<img src="src/WoeUSB/data/woeusb-logo.png" alt="WoeUSB logo" width="160" />
<br />
<em>A modern, safe, high-performance Linux application to create bootable Windows USB installers from ISO images or DVDs.</em>
</div>

---

`woeusb-rs` is a complete rewrite of WoeUSB in Rust, architected with a decoupled core engine, a versatile CLI utility with simulation mode, and a native GNOME/Libadwaita desktop GUI.

## Highlights & Features

- **Blazing Fast & Efficient**: Native Rust disk inspection and buffered multi-megabyte chunked file transfer with real-time throughput metrics.
- **Fail-Safe System Disk Protection**: Unconditionally verifies mount points and swap tables, preventing accidental destruction of your operating system (`/`, `/home`, etc.).
- **Zero Runtime Internet Dependency**: Embedded Rufus `uefi-ntfs.img` partition image directly within the binary for offline NTFS UEFI booting.
- **Automatic 4GiB Limit Handling**: Automatically detects when `install.wim` exceeds the FAT32 4GiB boundary and configures NTFS + UEFI:NTFS partitions.
- **Modern Split Privilege Architecture**:
  - Unprivileged **GTK4 / Libadwaita** GUI runs natively under Wayland and X11 as your normal desktop user.
  - Privileged disk manipulation runs safely in a dedicated worker process authorized via **Polkit** (`pkexec`).
- **Complete `--dry-run` Mode**: Test and verify the full partitioning and copying workflow without touching physical disks.
- **Dual CLI Compatibility**: Modern subcommands (`flash`, `list`, `info`) alongside 100% backward compatibility for legacy WoeUSB scripts (`--device`, `--partition`).

---

## Installation

### 1. Arch Linux / CachyOS / Manjaro

#### Via AUR Helper (`paru` or `yay`):
```bash
paru -S woeusb-rs-git
# or
yay -S woeusb-rs-git
```

#### Manual Build with `makepkg` & `pacman`:
```bash
git clone https://github.com/rinme/woeusb-rs.git
cd woeusb-rs
makepkg -si
```

#### Uninstalling on Arch:
```bash
sudo pacman -R woeusb-rs-git
```

---

### 2. Generic Linux Installation (From Source)

#### Dependencies
- Runtime utilities: `parted`, `wipefs`, `dosfstools` (`mkfs.vfat`), `ntfs-3g` (`mkntfs`), `grub2-common` / `grub-pc-bin`, `7z` (`p7zip`)
- Build dependencies: `rust` / `cargo` (1.80+), `gtk4`, `libadwaita-1`, `pkg-config`, `make`

```bash
git clone https://github.com/rinme/woeusb-rs.git
cd woeusb-rs

# Compile release binaries
make build
# or: cargo build --release

# Install binaries, desktop launcher, icons, and Polkit policy
sudo make install
```

#### Uninstalling:
To completely remove all installed binaries, desktop launchers, icon assets, and Polkit policies:
```bash
sudo make uninstall
```

---

## Command-Line Usage (`woeusb`)

### 1. Flash a Windows ISO to a USB Drive

```bash
# Modern subcommand syntax
sudo woeusb flash Win11_English_x64.iso /dev/sdb

# With simulation / dry-run mode (safe to test without writing to disk)
woeusb flash --dry-run Win11_English_x64.iso /dev/sdb

# Specify custom volume label and skip legacy GRUB
sudo woeusb flash --label "WIN11" --skip-grub Win11_English_x64.iso /dev/sdb
```

### 2. List Connected Storage Devices

```bash
woeusb list

# Include non-removable internal drives
woeusb list --all
```

### 3. Inspect a Windows ISO Image

```bash
woeusb info Win11_English_x64.iso
```

### 4. Legacy Syntax Compatibility

Existing scripts and commands continue to work out of the box:

```bash
sudo woeusb --device Win10.iso /dev/sdb
sudo woeusb --partition Win10.iso /dev/sdb1
```

---

## Graphical User Interface (`woeusbgui`)

Launch the modern Libadwaita interface from your application menu or run:

```bash
woeusbgui
```

The GUI runs in user space and automatically requests Polkit privilege elevation when initiating the burn process.

---

## Architecture

The project is organized as a Cargo workspace:

- [`crates/woeusb-core`](crates/woeusb-core): Core library for drive detection, safety analysis, ISO inspection, file copy streaming, and partition orchestration.
- [`crates/woeusb-cli`](crates/woeusb-cli): Terminal application (`woeusb`) with `indicatif` progress bars, safety prompts, and JSON-streaming worker mode.
- [`crates/woeusb-gui`](crates/woeusb-gui): Native desktop GUI (`woeusbgui`) written in GTK4 + Libadwaita.
- [`packaging/aur`](packaging/aur): AUR-ready packaging files (`PKGBUILD`, `.SRCINFO`).

---

## License

Distributed under the terms of the [GNU General Public License v3 or later](COPYING).
