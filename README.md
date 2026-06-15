# puppy — GitHub package installer

```
   _   _   _   _   _   _  
  / \ / \ / \ / \ / \ / \ 
 ( p | u | p | p | y | ! )
  \_/ \_/ \_/ \_/ \_/ \_/ 
```

puppy is a small Linux tool that clones software directly from GitHub, detects the project type, builds it when possible, and installs binaries into `~/.puppy/bin` while keeping sources under `~/.puppy/src`.

---

## Installation

Prerequisites:

- Git
- Rust toolchain (for building `puppy` itself and for installing Cargo projects)
- Optional: `make`, `cmake`, `go`, `npm` depending on what you install

1) Build from source (developer / recommended):

```bash
# clone this repo (if not already)
 git clone <your-repo-url> puppy
 cd puppy
 # build the release binary
 cargo build --release
```

2) Run directly with Cargo (quick test):

```bash
# example: install sharkdp/bat for testing
 cargo run -- install sharkdp/bat
```

3) Run the compiled binary:

```bash
 ./target/release/puppy install sharkdp/bat
```

4) Add `~/.puppy/bin` to your PATH so installed binaries are available:

```bash
 echo 'export PATH="$HOME/.puppy/bin:$PATH"' >> ~/.profile
 source ~/.profile
```

---

## Commands & Usage

Install a GitHub repo (owner/repo or GitHub URL):

```bash
 puppy install owner/repo
 # example
 puppy install sharkdp/bat
```

List installed packages:

```bash
 puppy list
```

Uninstall a package (removes source dir and the binary):

```bash
 puppy uninstall owner/repo
```

Show information about puppy:

```bash
 puppy about
```

Help:

```bash
 puppy help
```

---

## How it works

1. Clone into `~/.puppy/src/<owner>-<repo>`
2. Detect project type by presence of `Cargo.toml`, `go.mod`, `CMakeLists.txt`, `Makefile`, or `package.json`
3. Run the appropriate build/install command:

- Rust: `cargo install --path . --root ~/.puppy`
- Go: `go build -o ~/.puppy/bin/<name>`
- CMake: `cmake` + `cmake --build ... --target install`
- Make: `make` and `make install PREFIX=~/.puppy`
- Node: `npm install --prefix ~/.puppy`

4. Place files/binaries in `~/.puppy/bin`

---


- Ensure required toolchains are installed for the projects you install.
- If `make install` fails, inspect the sources in `~/.puppy/src/<owner>-<repo>` and run the appropriate install steps manually.
- Node-installed binaries may be under `~/.puppy/lib/node_modules/.bin` — add it to `PATH` if necessary.

## Contributing

Contributions are welcome. Suggested next features:

- `puppy upgrade` to rebuild/update installed packages
- `puppy search` to find repos (GitHub API integration)
- manifest tracking of installed packages for safer `uninstall`

## License

Include a `LICENSE` file (MIT or Apache-2.0 recommended).
