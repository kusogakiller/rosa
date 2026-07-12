# RosaClient User Manual

An unofficial TUI client for **zinro.net**—a Japanese Werewolf game platform.

> **New to Rosa?** Start with [Quick Start] to get running in minutes.

> 🇯🇵 日本語をご利用の方へ  
> 日本語版ドキュメントはこちらです: [README.ja.md](README.ja.md)

---

## 1. Overview

### What Rosa Does

Rosa is a lightweight **Terminal User Interface (TUI)** client for [zinro.net](https://zinro.net). It lets you browse and send chat messages directly from your terminal without opening a browser.

Core functionality:

- **Poll server every 2 seconds** (configurable) to fetch chat messages and player info (`src/main.rs:711`, `src/api.rs:18`)
- **Send messages** to the global chat channel (ALL) (`src/api.rs:223`)
- **View participants** and room metadata—name, scene, date, capacity (`src/ui.rs:350`)
- **Navigate** with Vim-like keybindings and a 4-panel layout (Explorer / Chat / Context / Terminal)
- **Show pending state** for messages being sent with a temporary "sending…" indicator (`src/main.rs:564`)
- **Display connection status** (online / offline) in real-time

### Why Use Rosa?

- **Stay in the terminal**—no browser tab needed
- **Keyboard-first workflow**—fast, vim-like controls
- **Lightweight**—minimal dependencies, minimal overhead
- **Open source**—review and contribute to the code

### Disclaimer

Rosa is an **unofficial, community-built client** for zinro.net. Use it at your own discretion.

---

## 2. Requirements

### Supported Platforms

Rosa is written in Rust and runs anywhere Rust and `crossterm` (a terminal control library) work:

- Windows
- Linux  
- macOS

### Dependencies

To build and run Rosa, you'll need:

- **Rust toolchain** (including `cargo`)
  - Edition 2024 (`Cargo.toml:4`). Stable Rust recommended.
- **Internet connection** (to reach `https://zinro.net`)
- **Valid session key** (explained in Setup)

### Key Libraries

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `reqwest` | HTTP client |
| `ratatui` / `crossterm` | TUI rendering & input |
| `serde` / `serde_json` | JSON deserialization |
| `dotenvy` | Environment variables from `.env` |
| `anyhow` | Error handling |
| `tracing` / `tracing-subscriber` | Structured logging |

### Recommended Setup

- **Nerd Fonts–compatible terminal** (optional but recommended)
  - Rosa uses Unicode glyphs (👤, etc.) from the Nerd Fonts symbol set (`src/ui.rs`). Without proper font support, you'll see □ or ? instead.
- **True Color (24-bit) terminal** (optional but recommended)
  - Colors use RGB values (`src/ui.rs:17+`). True Color terminals render these beautifully.

---

## 3. Installation

Rosa has no installer or pre-built packages. **You'll build it from source.**

### Step 1: Install Rust

If you don't have Rust, grab it from the official installer:

**→ <https://rustup.rs>** (Windows / macOS / Linux)

After installing, open a fresh terminal and verify:

```bash
cargo --version
```

You should see something like `cargo 1.xx.x ...`.

### Step 2: Get the Source

Clone or download the Rosa repository. Navigate to the project directory:

**Linux / macOS:**

```bash
cd /path/to/rosa
```

**Windows (PowerShell):**

```powershell
cd C:\path\to\rosa
```

### Step 3: Build

Create a release binary:

```bash
cargo build --release
```

On success, your executable is ready:

- **Linux / macOS:** `target/release/rosa`
- **Windows:** `target/release/rosa.exe`

### Platform Notes

The `cargo` commands themselves are identical across all platforms. The only differences are:

- Path separators (Windows: `\`, Unix: `/`)
- Executable extension (Windows: `.exe`, Unix: none)
- Terminal/shell (Windows: PowerShell, Unix: any shell)

**Everything else is the same.**

---

## 4. Initial Setup

### Configuration: Environment Variables

Rosa requires a **`SESSION_KEY`** environment variable at startup (`src/main.rs:703`):

```rust
let session_key = std::env::var("SESSION_KEY").expect("SESSION_KEY is not set");
```

This key can come from two sources (via `dotenvy::dotenv()` in `src/main.rs:701`):

#### Option A: Using `.env` (Recommended)

Create a `.env` file in your project root (same directory as `Cargo.toml`):

```dotenv
SESSION_KEY=your_session_key_here
```

#### Option B: Export Environment Variable

**Linux / macOS:**

```bash
export SESSION_KEY="your_session_key_here"
cargo run --release
```

**Windows (PowerShell):**

```powershell
$env:SESSION_KEY = "your_session_key_here"
cargo run --release
```

### Getting Your Session Key

Your `SESSION_KEY` is a session token from zinro.net. To obtain it:

1. Open your browser and log in to <https://zinro.net>
2. Open Developer Tools (`F12` or `Cmd+Option+I`)
3. Go to **Application** / **Storage** → **Cookies**
4. Find the cookie named `session_key`
5. Copy its value

That's your session key. Alternatively, inspect the `Cookie` header in network requests—Rosa uses it here (`src/api.rs:194`):

```rust
Cookie: session_key=...
```

> **Security Notice:** Your `SESSION_KEY` is like a password. Never share it publicly or commit it to a repository. Keep `.env` in `.gitignore` if you use version control.

### First Run

1. Start Rosa (see "Running Rosa" below)
2. The screen switches to a TUI layout
3. You'll see the initial message: `RosaClient started` (`src/main.rs:122`)
4. Rosa begins polling the server every 2 seconds
5. If the connection succeeds, the status in the top-right shows **● online** (`src/ui.rs:170`)
6. If it fails, the status shows **● offline** and an error message appears in the chat panel (`src/main.rs:229`)

---

## 5. Getting Started

### Running Rosa

**Option A: Using `cargo run`**

```bash
cargo run --release
```

**Option B: Running the Binary Directly**

**Linux / macOS:**

```bash
./target/release/rosa
```

**Windows:**

```powershell
.\target\release\rosa.exe
```

> If `SESSION_KEY` is not set, you'll see: `thread 'main' panicked at 'SESSION_KEY is not set'`. See Setup section to fix this.

### Understanding the Layout

When Rosa starts, you'll see this layout (`src/ui.rs:95`):

```
┌ RosaClient  Explorer Chat Context Terminal          ● online ┐  ← Header
├──────────────┬─────────────────────────────────────────────┤
│ Rooms        │ Chat                                          │
│  Hallway      │  ● alice  hello everyone                      │
│  Study       │  ● bob    hey!                                 │
│              │  ● ...                                         │
│ Users        │                                               │
│  alice       │                                               │
│  bob         │                                               │
├──────────────┤                                               │
│ Context      ├───────────────────────────────────────────────┤
│              │ Terminal   ❯ chat message here                 │
│  online      │                                               │
│  Hallway     │                                               │
│  Day 3       │                                               │
│  3 / 10      │                                               │
├──────────────┴─────────────────────────────────────────────┤
│ NORMAL  Chat                              Hallway            │  ← Status bar
└─────────────────────────────────────────────────────────────┘
```

**Panels:**

- **Explorer** (top-left): Room and participant lists
- **Chat** (top-right): Incoming messages with sender and timestamp
- **Context** (bottom-left): Connection status, current room, game state
- **Terminal** (bottom-right): Your input line
- **Status bar** (bottom): Current mode, active panel, room name

### Modes (Vim Inspired)

Rosa has four modes. The current mode is shown in the bottom-left corner of the status bar.

| Mode | Display | Purpose |
|------|---------|---------|
| **Normal** | `NORMAL` | Move, scroll, and launch operations. The default. |
| **Insert** | `INSERT` | Type a message to send. |
| **Command** | `COMMAND` | Enter a colon command (`:q`, `:clear`, etc.). |
| **Search** | `SEARCH` | Enter a search query (`/text`). |

Press **Esc** to return to Normal mode from any other mode.

### Send Your First Message

1. Press **`i`** to enter Insert mode
2. Type your message:
   ```
   Hello everyone!
   ```
3. Press **`Enter`** to send
4. Your message appears with a temporary "sending…" indicator
5. Once the server confirms, the indicator disappears and your message is shown normally
6. Press **`Esc`** to return to Normal mode

> All messages are sent as public chat to everyone (ALL). There is a `whisper` function in the code (`src/api.rs:231`), but UI support for private messages is not yet implemented—this would be a great contribution!

---

## 6. Key Bindings & Commands

All operations happen after startup. **Rosa has no command-line arguments.** This section lists all Normal mode keybindings (`src/main.rs:367+`).

### Panel Navigation

| Key | Action | Notes |
|-----|--------|-------|
| `Tab` | Next panel (Explorer → Chat → Context → Terminal → ...) | `src/main.rs:503` |
| `Shift+Tab` | Previous panel | `src/main.rs:512` |
| `Ctrl+w` `h` | Left | Two-key combo: press Ctrl+w, then h |
| `Ctrl+w` `l` | Right | Two-key combo |
| `Ctrl+w` `j` | Down | Two-key combo |
| `Ctrl+w` `k` | Up | Two-key combo |

### Cursor Movement & Scrolling

| Key | Action |
|-----|--------|
| `j` or `↓` | One line down |
| `k` or `↑` | One line up |
| `Ctrl+d` | 8 lines down |
| `Ctrl+u` | 8 lines up |
| `Ctrl+f` | 16 lines down (full page) |
| `Ctrl+b` | 16 lines up (full page) |
| `gg` | Jump to start |
| `G` | Jump to end |

In the Explorer panel, use `h` / `←` to switch to **Rooms** and `l` / `→` to switch to **Users** (`src/main.rs:529`).

### Entry, Search & Mode Changes

| Key | Action |
|-----|--------|
| `i`, `a`, `A`, `I`, `o`, `O` | Enter Insert mode (all behave identically) |
| `:` | Enter Command mode |
| `/` | Enter Search mode |
| `n` | Next search result |
| `N` | Previous search result |
| `Esc` | Return to Normal mode (or cancel pending command) |
| `q` | Quit application |

### Delete Messages (Chat Panel)

| Key | Action |
|-----|--------|
| `x` | Delete the message at cursor |
| `dd` | Delete the message at cursor |

> **Important:** Deletion only affects your **local display**. It does not delete the message from the server (`src/main.rs:318`). Think of it as a personal "hide" function.

### Insert Mode Editing

| Key | Action |
|-----|--------|
| Text keys | Add characters to input |
| `Backspace` | Delete one character |
| `Enter` | Send your message |
| `Esc` | Return to Normal mode (discard draft) |

### Colon Commands (Command Mode)

Press `:`, type a command, and press `Enter` (`src/main.rs:602`). Available commands:

| Command | Action | Example |
|---------|--------|---------|
| `:q` | Exit Rosa | `:q` `Enter` |
| `:quit` | Exit Rosa (alias for `:q`) | `:quit` `Enter` |
| `:clear` | Clear all chat messages from your display | `:clear` `Enter` |

Any unrecognized command returns you to Normal mode silently (`src/main.rs:612`).

### Search (Search Mode)

Press `/`, type a search term, and press `Enter` (`src/main.rs:631`).

- **Search scope:** Both usernames and message text
- **Case sensitivity:** Case-insensitive (`src/main.rs:346`)
- **Navigation:** Use `n` to go to the next match, `N` for the previous match
- **Result:** Chat panel jumps to the first match with your cursor positioned there

Example:

```
/hello
```

---

## 7. Troubleshooting

### Error: "SESSION_KEY is not set"

**Problem:** The app exits immediately with this message.

**Cause:** Rosa can't find your session key (`src/main.rs:703`).

**Solution:**

1. Create a `.env` file in your project root with `SESSION_KEY=your_key`
2. OR set the environment variable before running (see Setup section)
3. Verify the file exists and the key is correct

---

### Status shows "● offline" with red error messages

**Problem:** Rosa can't reach the server.

**Cause:** Network issue, invalid/expired session key, or server problem.

**Troubleshooting:**

1. Check your internet connection
2. Open <https://zinro.net> in a browser—does it load?
3. Verify your `SESSION_KEY` is current (may be expired; re-login to zinro.net and grab a fresh key)
4. Read the red error message in the chat panel for clues (`src/main.rs:229`)
5. Check if zinro.net's server is operational

---

### Symbols show as "□" or "?"

**Problem:** Rosa displays boxes instead of icons.

**Cause:** Your terminal font doesn't support Unicode symbols from Nerd Fonts.

**Solution:** Install and configure a Nerd Fonts font (e.g., Fira Code Nerd Font, Jetbrains Mono Nerd Font). Many terminals let you change the font in preferences.

---

### Colors look wrong or washed out

**Problem:** The color palette looks dull or unsupported.

**Cause:** Your terminal doesn't support True Color (24-bit RGB).

**Solution:** Use a modern terminal that supports True Color:
- **Linux:** `gnome-terminal`, `Konsole`, `Terminator`, or `iTerm2`
- **macOS:** `iTerm2`, or the built-in Terminal (enable RGB Color in Preferences)
- **Windows:** Windows Terminal (built-in on Windows 10+)

---

### My message takes a while to appear

**Problem:** You sent a message, but it doesn't show up immediately.

**Cause:** Rosa polls the server every 2 seconds (`src/api.rs:18`). Messages take time to round-trip. Also, the API call itself is rate-limited (`src/api.rs:157`).

**Expected behavior:** You'll see your message with a "sending…" indicator, then it's replaced with the confirmed message once the server responds. Total delay is usually 2–4 seconds.

**What to do:** This is by design. Relax and wait for the chat panel to update.

---

### Rosa won't close / screen looks corrupted

**Problem:** Quit commands don't work or the terminal display is messed up.

**Solution:**

1. Press **`q`** in Normal mode, or
2. Type **`:q`** and press `Enter`

Rosa should exit cleanly and restore your terminal. If the display is corrupted, you can always run `reset` or `clear` in your terminal to restore it.

---

## 8. For Developers

### Polling Interval

- Server polling happens every **2000 ms (2 seconds)** by default (`src/main.rs:711`)
- The Context panel displays the current interval (`src/ui.rs:414`)
- **Limitation:** This is hardcoded. There's no config file or env var to change it yet.
- **To customize:** Edit the source and rebuild

### Automation & Integration

Rosa is built as an **interactive TUI**, not a batch tool. There's no non-interactive mode or piping support.

If you want to **automate messages** (e.g., from a script):

- Use the `ApiClient` directly from `src/api.rs` (`send_message`, etc.)
- Write your own Rust program that imports and calls these functions
- (Rosa itself has no built-in scripting interface)

### API & Network Details

| Item | Value |
|------|-------|
| **Server** | `https://zinro.net` (hardcoded in `src/api.rs:16`) |
| **Message fetch** | `GET /m/api/?mode=message` (`src/api.rs:239`) |
| **Player fetch** | `GET /m/api/?mode=players` (`src/api.rs:262`) |
| **Send message** | `POST /m/player.php?mode=message&to_user=ALL&message=...` (`src/api.rs:205`, `src/api.rs:223`) |
| **Auth** | Cookie header: `session_key=<SESSION_KEY>` (`src/api.rs:194`) |

### Common Rust Commands

```bash
cargo build          # Debug build
cargo build --release # Optimized release binary
cargo run --release   # Build & run in one step
cargo fmt             # Auto-format code
cargo clippy          # Lint suggestions
```

---

## Contributing

Found a bug? Have an idea? Contributions are welcome! Some ideas for the future:

- [ ] Configurable polling interval (CLI flag or config file)
- [ ] Private message / whisper UI support
- [ ] Message reactions or emotes
- [ ] Color themes / customizable palette
- [ ] Multi-room support
- [ ] Better error messages and recovery
- [ ] Unit tests for API client

---

## License

MIT License

Copyright (c) 2026 negiradomoti

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

## Questions?

- Open an issue on GitHub
- Read the source code—it's well-structured and readable
- Ask the developer

Enjoy your Rosa experience!