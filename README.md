# Rusty Vim

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)

A modal, Vim-like text editor built from scratch in **~1500 lines of Rust**, using only the termios crate for terminal interaction.

Rusty Vim started as a project to understand terminal applications (initially inspired by the Kilo editor tutorial) and evolved into implementing core Vim features and motions. It focuses on leveraging Rust's safety features while recreating the efficient, keyboard-centric editing experience Vim users appreciate.

## Features

Rusty Vim implements some of Vim's core functionality

**Editing Modes:**

- **Normal Mode:** For navigation and issuing commands (default mode).
- **Insert Mode:** For typing text directly into the buffer.
- **Command Mode:** For entering Ex commands (like `:w`, `:q`).

**Navigation (Normal Mode):**

- **Basic Motion:** `h` , `j`, `k`, `l`.
- **Word Motion:** `w` (next word start), `W` (next WORD start).
- **Line Motion:**
  - `0` (start of line), `^` (first non-whitespace character).
  - `$` (end of line).
  - `gg` (go to first line), `G` (go to last line).
  - `<N>G` or `<N>gg` (go to line N ).
- **Paragraph Motion:** `{` (previous paragraph), `}` (next paragraph).

**Editing (Normal Mode):**

- **Entering Insert Mode:**
  - `i` (insert before cursor), `I` (insert at start of line).
  - `a` (append after cursor), `A` (append at end of line).
  - `o` (open line below), `O` (open line above).
- **Deletion:**
  - `x` (delete character under cursor).
  - Uses the `d` operator combined with motions:
    - `dd` (delete current line).
    - `dw` (delete word).
    - `d$` (delete to end of line).
    - `d^` (delete to first non-whitespace).
    - `dG` (delete to end of file).
    - `dgg` (delete to start of file).
    - `dh`, `dj`, `dk`, `dl` (delete based on direction).
  - _Repeat counts work with deletions (e.g., `d5w`, `2dd`)._

**Editing (Insert Mode):**

- Standard text entry.
- `Backspace` key support.
- `Enter` key for newlines (with basic auto-indent).
- `Tab` key inserts spaces (currently hardcoded to 4).
- `Esc` to return to Normal Mode.

**Command Mode (`:`):**

- **File Operations:**
  - `:w` (write/save file).
  - `:w <filename>` (write to a specific file).
  - `:w!` (force write, ignoring modifications).
  - `:q` (quit current buffer if all changes is written).
  - `:q!` (force quit, discard changes).
  - `:wq` (write and quit).
  - `:wq!` (force write and quit).
- **Buffer Management:**
  - `:bn` (next buffer).
  - `:bp` (previous buffer).
  - `:b <N>` (go to 0 indexed buffer ).
  - `:b <N+1>` (open a new empty buffer if N is the current last index).

**Buffer Handling:**

- Open multiple files from the command line (`rusty-vim file1 file2`).
- Handles empty buffers for new files.
- Tracks unsaved changes (`is_changed` flag).
- Checks for external file modifications before non-forced saves (`:w`).

**Terminal UI:**

- Custom UI rendering using ANSI escape codes.
- `Nord`-inspired color theme.
- **Status Line:** Displays current mode, filename, modified status (_implicitly via save checks_), cursor position (line:col).
- **Command Line:** Shows typed commands and status messages (e.g., save confirmation, errors).
- Line numbers displayed on the left.
- Cursor shape changes based on mode (Block for Normal/Command, I-Beam for Insert).

## Prerequisites

- **Rust:** Ensure you have [rust](https://www.rust-lang.org/tools/install) Installed in your system.
- **A Unix-like Terminal:** Relies on `termios` for terminal control (Linux, macOS).

## Installation & Usage

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/dijith-481/rusty-vim.git
    cd rusty-vim
    ```

2.  **Build and Run:**

    - **To run directly (compiles and runs):**

      ```bash
      # Open an empty buffer
      cargo run

      # Open specific files
      cargo run  foo1.txt path/to/foo.rs
      ```

    - **To build an optimized release binary:**
      ```bash
      cargo build --release
      # The binary will be in ./target/release/rusty-vim
      ./target/release/rusty-vim [optional_file ...]
      ```

## Acknowledgements

- Inspired by Vim.
- Initial structure influenced by the [Kilo editor tutorial](https://viewsourcecode.org/snaptoken/kilo/).
- Built with the `termios` crate for terminal control.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
