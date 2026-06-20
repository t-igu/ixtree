# ixtree

A fast and configurable directory tree viewer written in Rust.  
`ixtree` provides a clean tree output with ignore rules defined in `.ixtree.toml`  
or via command-line options.

---

## ✨ Features

- **Ignore rules** via `.ixtree.toml`
- **Explicit config file** via `--config`
- **Additional ignore patterns** via `-I`
- **Directory-first sorting** (`--dirsfirst`)
- **Property display** (`-p`)
- **Safe & predictable behavior**  
  - No parent-directory config search  
  - Only the explicitly provided config file or the current directory is used

---

## 📦 Installation

```bash
cargo build --release
```

Binary will be located at:

```
target/release/ixtree
```

---

## 🚀 Usage

```bash
ixtree [PATH] [OPTIONS]
```

### Examples

```bash
# Show current directory
ixtree .

# Show parent directory
ixtree ..

# Show properties
ixtree -p .

# Add ignore patterns
ixtree -I target -I "*.log" .

# Use explicit config file
ixtree --config ./config/ixtree.toml .
```

---

## ⚙ Configuration

`ixtree` reads ignore patterns from `.ixtree.toml`  
located in the **current working directory**.

```toml
[ignore]
patterns = [
    "target",
    "node_modules",
    "*.log"
]
```

You can override this using:

```bash
ixtree --config ./myconfig.toml .
```

---

## 🧪 Example Output

```
C:/workspace/myproject/
├── src
│   ├── main.rs
│   └── walker.rs
├── Cargo.toml
└── README.md
```

---

## 📄 License

MIT



