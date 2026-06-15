# puppy — GitHub package installer

![GitHub stars](https://img.shields.io/github/stars/risotto-slackware/puppy?style=flat)
![GitHub forks](https://img.shields.io/github/forks/risotto-slackware/puppy?style=flat)
![License](https://img.shields.io/github/license/risotto-slackware/puppy?style=flat)
![Platform](https://img.shields.io/badge/platform-linux-blue)
![Rust](https://img.shields.io/badge/rust-1.70+-orange)



puppy is a small Linux tool that clones software directly from GitHub, detects the project type, builds it when possible, and installs binaries into `~/.puppy/bin` while keeping sources under `~/.puppy/src`.

---

## Installation

> [!IMPORTANT]
> Make sure required dependencies are installed before using puppy.

### Prerequisites
- Git
- Rust toolchain (for building `puppy` itself and installing Cargo projects)
- Optional: `make`, `cmake`, `go`, `npm` depending on what you install

---

### 1) Build from source (recommended)

```bash
git clone <your-repo-url> puppy
cd puppy
cargo build --release
```

---

### 2) Run directly with Cargo (quick test)

```bash
cargo run -- install sharkdp/bat
```

---

### 3) Run compiled binary

```bash
./target/release/puppy install sharkdp/bat
```

---

### 4) Add to PATH

> [!TIP]
> Allows installed binaries to be used globally.

```bash
echo 'export PATH="$HOME/.puppy/bin:$PATH"' >> ~/.profile
source ~/.profile
```

---

## Commands & Usage

### Install a GitHub repo

```bash
puppy install owner/repo
puppy install sharkdp/bat
```

---

### List installed packages

```bash
puppy list
```

---

### Uninstall a package

```bash
puppy uninstall owner/repo
```

---

### About puppy

```bash
puppy about
```

---

### Help

```bash
puppy help
```

---

## How it works

> [!IMPORTANT]
> puppy automatically detects project type and builds accordingly.

1. Clone into:
```
~/.puppy/src/<owner>-<repo>
```

2. Detect project type:
- Cargo.toml
- go.mod
- CMakeLists.txt
- Makefile
- package.json

3. Build system used:

- Rust:
```bash
cargo install --path . --root ~/.puppy
```

- Go:
```bash
go build -o ~/.puppy/bin/<name>
```

- CMake:
```bash
cmake
cmake --build . --target install
```

- Make:
```bash
make
make install PREFIX=~/.puppy
```

- Node:
```bash
npm install --prefix ~/.puppy
```

4. Output:
```
~/.puppy/bin
```

---

## Warnings

> [!WARNING]
> Some projects require additional dependencies before building.

> [!CAUTION]
> External install scripts may modify system-like paths. Prefer `~/.puppy`.

> [!WARNING]
> Node binaries may appear in:
```
~/.puppy/lib/node_modules/.bin
```

---

## Contributing

> [!TIP]
> Contributions are welcome.

Suggested improvements:
- `puppy upgrade`
- `puppy search` (GitHub API integration)
- install manifest tracking for safer uninstall

---

## License

Check the repository for the license file.
