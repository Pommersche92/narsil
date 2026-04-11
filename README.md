<div align="center">
  <img src="icon.png" alt="narsil icon" width="128" /><br><br>
  <h1>Narsil</h1>
</div>

> A terminal-based system resource monitor written in Rust — fast, readable, and GPU-aware.

Named after the reforged sword of Aragorn, **narsil** is built to be sharper than the tools that came before it. It targets developers and power users who live in the terminal and need real-time system insight without leaving it.

> **Platform support** — narsil runs on **Linux**, **Windows**, and **macOS**. The GPU tab (AMD/NVIDIA) is Linux-only for now; all other tabs work on every supported OS.

---

## 📸 Screenshot

![narsil screenshot](screenshot.png)

---

## ✨ Features

### Current scope (v0.1)

| Tab | What you see | Platform |
|-----|-------------|----------|
| 🗺️ **Overview** | CPU gauge, RAM gauge, live RX/TX sparklines, top processes (fills available height) | all |
| 🧠 **CPU** | Global usage history chart (Braille), per-core gauges with colour-coded load | all |
| 💾 **Memory** | RAM + Swap history charts, GiB usage gauges | all |
| 🌐 **Network** | Combined RX/TX history chart, per-direction current throughput | all |
| 💿 **Disks** | Per-partition usage bars at fixed height, scrollable when partitions exceed the terminal | all |
| 🔬 **Processes** | Process table sorted by CPU, scrollable, fills available height | all |
| 🎮 **GPU** | Per-GPU cards with utilisation + VRAM history charts, gauges, temperature and power draw | **Linux only** |

### 🔑 Key behaviours

- 🎨 **Split-colour gauges** — the percentage label rendered inside every gauge automatically inverts its colour character-by-character at the fill boundary so it is always readable, even when the bar is exactly at 50%.
- 📜 **Scroll indicators** — any panel that cannot display all items at once shows `▲`/`▼`/`▲▼` in its title.
- 📐 **Dynamic sizing** — all panels adapt to the current terminal dimensions; no hard-coded row counts.
- ⚡ **Configurable refresh rate** — pass `--interval <ms>` on startup (default 1 000 ms) to tune between low-latency and low-CPU usage; key events are processed between ticks with zero busy-waiting.
- ⌨️ **Keyboard-first navigation**: `Tab` / `Shift+Tab` wrap-around tab switching; `1`–`6` direct jump (`1`–`7` on Linux); `j`/`k` or arrow keys for scrolling; `q` or `Ctrl-C` to quit.
- 💬 **Status bar** — persistent one-line keybinding reference at the bottom, context-aware per tab.

---

## 🚀 Installation

### Prerequisites

- **Linux**, **Windows 10+**, or **macOS 12+**
- GPU tab requires Linux with standard `/sys` mounts and `amdgpu` (AMD) or NVIDIA proprietary drivers
- Rust toolchain ≥ 1.85 — needed only for `cargo install` or source builds (`rustup update stable`)

### Official release channels

| Channel | Platforms | Standard | NVIDIA variant |
|---------|-----------|----------|----------------|
| **crates.io** | all | `cargo install narsil` | `cargo install narsil --features nvidia` |
| **AUR** | Arch Linux | `narsil` · `narsil-bin` | `narsil-nvidia` · `narsil-nvidia-bin` |
| **AppImage** | Linux x86_64 | `narsil-{ver}-x86_64.AppImage` | `narsil-nvidia-{ver}-x86_64.AppImage` |
| **Windows zip** | Windows x86_64 | `narsil-{ver}-x86_64-windows.zip` | `narsil-nvidia-{ver}-x86_64-windows.zip` |
| **Linux tarball** | Linux x86_64 | `narsil-{ver}-x86_64.tar.gz` | `narsil-nvidia-{ver}-x86_64.tar.gz` |
| **Source** | all | `cargo build --release` | `cargo build --release --features nvidia` |

AppImage, Windows zip, and Linux tarball are attached to every [GitHub release](https://github.com/Pommersche92/narsil/releases).

### crates.io

```bash
cargo install narsil                      # standard
cargo install narsil --features nvidia    # with NVIDIA GPU support (Linux only)
```

### AUR (Arch Linux)

Four packages are published to the AUR:

| Package | Type | Description |
|---------|------|-------------|
| [`narsil`](https://aur.archlinux.org/packages/narsil) | source | standard, compiled from the crates.io source tarball |
| [`narsil-nvidia`](https://aur.archlinux.org/packages/narsil-nvidia) | source | like `narsil` but built with `--features nvidia` |
| [`narsil-bin`](https://aur.archlinux.org/packages/narsil-bin) | binary | standard, installs prebuilt tarball from GitHub Releases |
| [`narsil-nvidia-bin`](https://aur.archlinux.org/packages/narsil-nvidia-bin) | binary | NVIDIA variant, installs prebuilt tarball from GitHub Releases |

```bash
yay -S narsil              # standard, built from source
yay -S narsil-nvidia       # with NVIDIA GPU support, built from source
yay -S narsil-bin          # standard, prebuilt binary
yay -S narsil-nvidia-bin   # with NVIDIA GPU support, prebuilt binary
```

### AppImage (Linux x86_64)

Download from the [latest GitHub release](https://github.com/Pommersche92/narsil/releases/latest), make executable and run:

```bash
chmod +x narsil-*-x86_64.AppImage
./narsil-*-x86_64.AppImage
```

Two variants per release: `narsil-{version}-x86_64.AppImage` (standard) and `narsil-nvidia-{version}-x86_64.AppImage` (with NVIDIA support).

### Windows (x86_64)

Download `narsil-{version}-x86_64-windows.zip` or `narsil-nvidia-{version}-x86_64-windows.zip` from the [latest GitHub release](https://github.com/Pommersche92/narsil/releases/latest), extract, and run `narsil.exe` in PowerShell or cmd.

> The GPU tab is not compiled into the Windows build.

### Linux tarball (x86_64)

Download from the [latest GitHub release](https://github.com/Pommersche92/narsil/releases/latest):

```bash
tar xzf narsil-{version}-x86_64.tar.gz
./narsil-{version}/narsil
```

Both standard and NVIDIA variants are available.

### Build from source

```bash
git clone https://github.com/Pommersche92/narsil
cd narsil
cargo build --release
./target/release/narsil        # Linux / macOS
.\target\release\narsil.exe    # Windows

# With NVIDIA GPU support (Linux only):
cargo build --release --features nvidia
```

---

## 🎮 GPU support matrix

> GPU monitoring is **Linux-only**. On Windows and macOS the GPU tab is not compiled in; all other tabs work normally.

| Vendor | Driver | Detected | Utilisation | Memory | Temperature | Power |
|--------|--------|----------|-------------|--------|-------------|-------|
| 🔴 AMD discrete | `amdgpu` | ✅ | ✅ `gpu_busy_percent` | ✅ VRAM | ✅ hwmon | ✅ hwmon |
| 🔴 AMD iGPU (APU) | `amdgpu` | ✅ | ✅ | ⚠️ GTT (shared RAM) | ✅ | ✅ |
| 🟢 NVIDIA | proprietary + `--features nvidia` | ✅ | ✅ NVML | ✅ NVML | ✅ NVML | ✅ NVML |
| 🔵 Intel iGPU | `i915` / `xe` | ❌ | — | — | — | — |
| 🔵 Intel Arc discrete | `xe` | ❌ | — | — | — | — |

> ⚠️ **AMD APU note**: the VRAM figures reflect GTT memory (system RAM dynamically assigned to the GPU), not dedicated video memory. The values are accurate but on-screen labels will stay as "VRAM" until the display is updated in a future release.

> 🗓️ **Intel note**: Intel GPU support is planned — see Roadmap below.

---

## ⌨️ Keybindings

| Key | Action | Platform |
|-----|--------|----------|
| `Tab` / `Shift+Tab` | Next / previous tab (wraps around) | all |
| `1` – `6` | Jump directly to tab | all |
| `7` | Jump to GPU tab | Linux only |
| `→` / `l` | Next tab | all |
| `←` / `h` | Previous tab | all |
| `↓` / `j` | Scroll down (Disks, Processes; + GPU on Linux) | all |
| `↑` / `k` | Scroll up | all |
| `q` / `Ctrl-C` | Quit | all |

---

## 🌍 Localisation

narsil ships with translations for four languages. The UI language is selected automatically from the OS locale and can be overridden at startup.

| Language | Code | Auto-detected from |
|----------|------|--------------------|
| English  | `en` | default fallback |
| German   | `de` | `LANG=de_DE.UTF-8`, `LANGUAGE=de`, … |
| French   | `fr` | `LANG=fr_FR.UTF-8`, `LANGUAGE=fr`, … |
| Spanish  | `es` | `LANG=es_ES.UTF-8`, `LANGUAGE=es`, … |

### Detection priority

1. `--lang <code>` CLI flag
2. `LANGUAGE` environment variable (colon-separated preference list, GNU extension)
3. `LC_ALL` → `LC_MESSAGES` → `LANG`
4. Native OS locale API (`GetUserDefaultLocaleName` on Windows, `CFLocale` on macOS)
5. English fallback

```bash
narsil --lang de    # force German
narsil --lang fr    # force French
```

### Adding a new language

Translation strings live in plain TOML files under `locales/` — no Rust knowledge needed to translate. See [locales/README.md](locales/README.md) for step-by-step instructions.

---

## 🏗️ Architecture

```
src/
├── main.rs               — terminal setup, raw-mode lifecycle, event + tick loop
├── app.rs                — App state dispatcher: calls each metrics::refresh on every tick
├── metrics/
│   ├── mod.rs            — HISTORY_LEN constant, push_history helper, re-exports
│   ├── cpu.rs            — CpuState, per-core + global history
│   ├── memory.rs         — MemState, RAM + swap
│   ├── network.rs        — NetState, RX/TX rates and history
│   ├── disks.rs          — DiskState, per-partition usage
│   ├── processes.rs      — ProcessEntry, top-100 by CPU
│   └── gpu/
│       ├── mod.rs        — GpuEntry, vendor dispatch
│       ├── amd.rs        — sysfs-based AMD metrics
│       └── nvidia.rs     — NVML-based NVIDIA metrics (feature-gated)
└── ui/
    ├── mod.rs            — draw() entry point
    ├── helpers.rs        — format_bytes, usage_color, scroll_indicator
    ├── statusbar.rs      — persistent keybinding bar
    ├── tab_bar.rs        — tab header row
    ├── widgets/
    │   └── split_gauge.rs — SplitGauge custom widget
    └── tabs/
        ├── overview.rs   — combined overview tab
        ├── cpu.rs
        ├── memory.rs
        ├── network.rs
        ├── disks.rs
        ├── processes.rs
        └── gpu.rs
```

Data flows in one direction:

```
app.on_tick()  →  App (shared state)  →  ui::draw()  →  ratatui frame
```

There is no async runtime; `crossterm::event::poll` provides the non-blocking event check.

---

## 🧪 Testing

The test suite lives in `src/tests/` and is compiled only in test builds (`#[cfg(test)]`). It covers the full public API: metric structs, refresh functions, UI helpers, and the `SplitGauge` widget.

```bash
cargo test
```

### Test coverage overview

| Module | What is tested |
|---|---|
| `tests::push_history` | Ring-buffer eviction, growth to capacity, length invariant |
| `tests::helpers` | `format_bytes` SI boundaries, `usage_color`/`_f64` thresholds, `scroll_indicator` states |
| `tests::cpu` | `CpuState::new` zeroed state & history dimensions; `refresh` valid range & history cap |
| `tests::memory` | `MemState::new` zeroed state; `refresh` `used ≤ total` + history cap |
| `tests::network` | `NetState::new` zeroed state; `refresh` history cap & rate consistency |
| `tests::disks` | `DiskState` field storage; `refresh` non-empty result, `used ≤ total`, non-empty names/mounts |
| `tests::processes` | `ProcessEntry` field storage; `refresh` ≤ 100 entries, CPU-descending sort, non-empty names |
| `tests::gpu` | `GpuEntry::new` zeroed fields, history lengths; `amd::refresh` smoke test & invariants | Linux only |
| `tests::split_gauge` | Ratio clamping, full/empty/half fill, label centring, block inner area, zero-size no-panic |

### Running with NVIDIA feature

```bash
cargo test --features nvidia
```

---

## 📦 Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI layout and widget rendering |
| `crossterm` | Cross-platform terminal control, raw mode, event stream |
| `sysinfo` | CPU, RAM, swap, network, disk, process data |
| `anyhow` | Ergonomic error handling |
| `nvml-wrapper` *(optional)* | NVIDIA GPU metrics via NVML |

---

## 🗺️ Roadmap

Items are loosely ordered by priority.

### 🔜 Near-term

- 🔵 **Intel GPU support** — utilisation via GT frequency ratio (`i915`/`xe` sysfs), LMEM for Intel Arc cards, temperature via hwmon; shown with appropriate caveats for iGPUs
- 🏷️ **AMD APU label fix** — distinguish GTT (shared) from dedicated VRAM and label accordingly
- ~~⏱️ **Configurable refresh rate** — CLI flag `--interval <ms>` to tune between low-latency and low-CPU usage~~ ✅
- 🎨 **Colour themes** — built-in dark/light/high-contrast theme switcher

### 🔧 Medium-term

- 🔬 **Per-process GPU attribution** — show which processes hold GPU memory (via NVML or `fdinfo` on the DRM driver)
- 🌡️ **Temperature history charts** — per-core CPU and GPU temperature sparklines, not just current values
- 💨 **Fan speed** — hwmon fan RPM display in the GPU card and a new thermal overview section
- 🌐 **Network per-interface breakdown** — drill-down view listing each interface (eth0, wlan0, lo…) separately with its own sparkline
- 💽 **Disk I/O throughput** — read/write MB/s per device, not just partition usage percentages
- 🔋 **Battery / power panel** — laptop-focused: charge level, rate of charge/discharge, estimated time remaining

### 🚀 Long-term / differentiators

- 📋 **Log tail panel** — a dedicated tab that tails systemd journal or a user-specified log file in real time, with regex highlight rules; something `htop` and `gotop` completely lack
- 🚨 **Alert rules** — user-defined thresholds (e.g. CPU > 90% for > 5 s, VRAM > 80%) that flash the affected panel border red and optionally send a desktop or webhook notification
- 🔌 **Plugin / script hooks** — allow arbitrary shell scripts or Rust dynamic libraries to provide custom metric panels, making narsil extensible without a fork
- 📼 **Session recording & replay** — record a metric session to a compact binary file and replay it later for post-mortem analysis
- 🖥️ **SSH-aware remote mode** — connect to a remote host via SSH and display its metrics locally in the same TUI, without needing narsil installed on the remote
- 🖱️ **Mouse support** — click tabs and scroll panels with the mouse alongside the existing keyboard navigation
- 📊 **Export** — one-shot `--json` / `--prometheus` output mode for integration with external dashboards (Grafana etc.)

---

## ⚖️ Comparison with existing tools

| Feature | `top` | `htop` | `gotop` | **narsil** |
|---------|-------|--------|---------|-----------|
| Language | C | C | Go | 🦀 **Rust** |
| GPU metrics | ❌ | ❌ | partial | **✅ AMD + NVIDIA (Linux)** |
| Braille charts | ❌ | ❌ | ✅ | **✅** |
| Per-char label inversion | ❌ | ❌ | ❌ | **✅** |
| Disk usage bars | ❌ | ❌ | ✅ | **✅** |
| Status bar with keybindings | ❌ | ❌ | ❌ | **✅** |
| Localised UI | ❌ | ❌ | ❌ | **✅ EN DE FR ES** |
| Log tail panel | ❌ | ❌ | ❌ | 🗓️ planned |
| Alert rules | ❌ | ❌ | ❌ | 🗓️ planned |
| Remote mode | ❌ | ❌ | ❌ | 🗓️ planned |
| Session replay | ❌ | ❌ | ❌ | 🗓️ planned |

---

## 📄 License

GPL-3.0 — see [LICENSE](LICENSE).

