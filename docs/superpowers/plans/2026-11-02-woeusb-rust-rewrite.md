# WoeUSB Rust Rewrite (`woeusb-rs`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite WoeUSB-ng in Rust as a high-performance, safe, modern Linux application featuring a shared core library (`woeusb-core`), a feature-rich CLI with `--dry-run` and legacy argument compatibility (`woeusb-cli`), and a native GTK4/Libadwaita GUI (`woeusb-gui`) communicating with a privileged worker via Polkit.

**Architecture:** A multi-crate Cargo workspace where `woeusb-core` provides native device discovery, safety validation, ISO inspection, file copying, and subprocess orchestration (`parted`, `mkfs.*`, `grub-install`). `woeusb-cli` provides an interactive terminal UI (`indicatif`) and a JSON-streaming `--worker` mode. `woeusb-gui` provides an unprivileged GTK4/Libadwaita desktop interface that spawns the worker via `pkexec`.

**Tech Stack:** Rust 2021, `clap` (derive), `indicatif`, `gtk4`, `libadwaita`, `serde`/`serde_json`, `thiserror`, `tracing`, `gettext-rs`.

**Spec:** Architectural blueprint agreed upon during the `/grill-me` session on 2026-11-02.

## Global Constraints

- Linux-first POSIX design.
- Rust edition 2021, compiling with standard stable rustc/cargo.
- Zero runtime internet download dependencies: embed `uefi-ntfs.img` into `woeusb-core` via `include_bytes!`.
- Unconditional safety guard: do not allow flashing disks containing mounted `/` or `/home`.
- Complete `--dry-run` mode to allow testing without physical disk writes.
- Git repository target: `https://github.com/rinme/woeusb-rs.git`.

---

### Task 1: Cargo Workspace & Core Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `crates/woeusb-core/Cargo.toml`
- Create: `crates/woeusb-core/src/lib.rs`
- Create: `crates/woeusb-core/src/error.rs`
- Create: `crates/woeusb-core/src/models.rs`
- Test: `crates/woeusb-core/tests/models_test.rs`

**Interfaces:**
- Produces: `TargetFilesystem`, `InstallMode`, `Drive`, `Partition`, `ProgressEvent`, `WoeError`, `Result<T>`.

- [ ] **Step 1: Write tests for data models and error conversions**
- [ ] **Step 2: Implement root `Cargo.toml` and `crates/woeusb-core/Cargo.toml`**
- [ ] **Step 3: Implement `error.rs` and `models.rs` with serialization support**
- [ ] **Step 4: Run `cargo test -p woeusb-core` and verify all pass**
- [ ] **Step 5: Git commit workspace scaffolding**

---

### Task 2: Core Device Discovery & System Disk Safety Guards

**Files:**
- Create: `crates/woeusb-core/src/device.rs`
- Create: `crates/woeusb-core/src/safety.rs`
- Modify: `crates/woeusb-core/src/lib.rs`
- Test: `crates/woeusb-core/tests/safety_test.rs`
- Test: `crates/woeusb-core/tests/device_test.rs`

**Interfaces:**
- Produces: `list_usb_drives() -> Result<Vec<Drive>>`, `inspect_device(path: &Path) -> Result<Drive>`, `check_device_safety(target_device: &Path) -> Result<SafetyReport>`.

- [ ] **Step 1: Write failing tests for mount table parsing and safety validation (e.g. blocking `/` and mounted swap)**
- [ ] **Step 2: Implement `/proc/mounts` and `/sys/block` parser in `safety.rs` and `device.rs`**
- [ ] **Step 3: Implement device model, size, and partition enumeration**
- [ ] **Step 4: Run `cargo test -p woeusb-core` to verify safety detection**
- [ ] **Step 5: Git commit device discovery and safety checks**

---

### Task 3: Core ISO Inspection & Bundled UEFI:NTFS Image

**Files:**
- Create: `crates/woeusb-core/src/iso.rs`
- Create: `crates/woeusb-core/src/assets.rs`
- Create: `crates/woeusb-core/assets/uefi-ntfs.img`
- Modify: `crates/woeusb-core/src/lib.rs`
- Test: `crates/woeusb-core/tests/iso_test.rs`

**Interfaces:**
- Produces: `inspect_iso(path: &Path) -> Result<IsoMetadata>`, `get_uefi_ntfs_image() -> &'static [u8]`, `is_fat32_compatible(iso: &IsoMetadata) -> bool`.

- [ ] **Step 1: Download/stage Rufus `uefi-ntfs.img` into `assets/`**
- [ ] **Step 2: Write tests for ISO inspection and `uefi-ntfs.img` embedding**
- [ ] **Step 3: Implement `iso.rs` to detect Windows version, `install.wim` / `install.esd`, and file sizes**
- [ ] **Step 4: Implement `assets.rs` exporting embedded byte arrays**
- [ ] **Step 5: Run tests and commit**

---

### Task 4: Core Engine Flashing & Simulation (`--dry-run`)

**Files:**
- Create: `crates/woeusb-core/src/engine.rs`
- Create: `crates/woeusb-core/src/copier.rs`
- Modify: `crates/woeusb-core/src/lib.rs`
- Test: `crates/woeusb-core/tests/engine_test.rs`

**Interfaces:**
- Produces: `Engine::new(config: FlashConfig)`, `Engine::run<F>(&self, on_progress: F) -> Result<()>`, where `F: Fn(ProgressEvent)`.

- [ ] **Step 1: Write test for dry-run engine workflow (ensuring step order: wipe -> partition -> format -> copy -> bootloader)**
- [ ] **Step 2: Implement partition creation (`parted`) and formatting (`mkfs.vfat`, `mkntfs`) with dry-run support**
- [ ] **Step 3: Implement buffered file copier with streaming throughput and cancel detection**
- [ ] **Step 4: Implement Windows 7 EFI workaround and legacy GRUB bootloader installation**
- [ ] **Step 5: Run tests and commit**

---

### Task 5: CLI Binary (`woeusb`)

**Files:**
- Create: `crates/woeusb-cli/Cargo.toml`
- Create: `crates/woeusb-cli/src/main.rs`
- Create: `crates/woeusb-cli/src/args.rs`
- Create: `crates/woeusb-cli/src/terminal.rs`
- Create: `crates/woeusb-cli/src/worker.rs`
- Test: `crates/woeusb-cli/tests/cli_args_test.rs`

**Interfaces:**
- Produces: `woeusb` executable with subcommands (`flash`, `list`, `info`), legacy flags (`--device`, `--partition`), and `--worker` mode.

- [ ] **Step 1: Write tests for CLI argument parsing (both modern subcommands and legacy flags)**
- [ ] **Step 2: Implement `args.rs` with `clap` derive**
- [ ] **Step 3: Implement `terminal.rs` using `indicatif` for human-readable progress and confirmation prompts**
- [ ] **Step 4: Implement `worker.rs` streaming JSON-lines progress events on stdout**
- [ ] **Step 5: Test CLI in `--dry-run` mode and commit**

---

### Task 6: GTK4 + Libadwaita GUI (`woeusbgui`)

**Files:**
- Create: `crates/woeusb-gui/Cargo.toml`
- Create: `crates/woeusb-gui/src/main.rs`
- Create: `crates/woeusb-gui/src/app.rs`
- Create: `crates/woeusb-gui/src/window.rs`
- Create: `crates/woeusb-gui/src/worker_client.rs`

**Interfaces:**
- Produces: `woeusbgui` executable presenting native Libadwaita window and launching `pkexec woeusb --worker ...`.

- [ ] **Step 1: Set up `crates/woeusb-gui` with GTK4 and Libadwaita dependencies**
- [ ] **Step 2: Implement window UI (ISO file chooser, drive dropdown, filesystem/boot options, progress bar)**
- [ ] **Step 3: Implement `worker_client.rs` to spawn `pkexec woeusb --worker ...` asynchronously and dispatch progress to GTK loop**
- [ ] **Step 4: Build and test GUI compilation**
- [ ] **Step 5: Commit GUI implementation**

---

### Task 7: Desktop Integration, Polkit Policy & Makefile

**Files:**
- Create: `Makefile`
- Modify: `miscellaneous/com.github.woeusb.woeusb-rs.policy`
- Modify: `miscellaneous/woeusbgui.desktop`
- Modify: `README.md`

**Interfaces:**
- Produces: Standard `make build`, `make test`, `make install` targets.

- [ ] **Step 1: Create Polkit policy for `woeusb` helper**
- [ ] **Step 2: Update desktop entry and icon assets**
- [ ] **Step 3: Create Makefile handling compilation and installation to `/usr/local`**
- [ ] **Step 4: Update README.md documenting Rust architecture and usage**
- [ ] **Step 5: Commit build and integration assets**

---

### Task 8: Verification & GitHub Push

- [ ] **Step 1: Run complete test suite: `cargo test --workspace`**
- [ ] **Step 2: Run `cargo clippy --workspace` to ensure clean code**
- [ ] **Step 3: Run `--dry-run` simulation on CLI**
- [ ] **Step 4: Push all commits to `origin master` (`https://github.com/rinme/woeusb-rs.git`)**
