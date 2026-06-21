// use regex::Regex;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use std::path::PathBuf;

#[derive(Serialize)]
struct Node {
    name: String,
    path: String,
    file_type: String,
    size: u64,
    modified: String,
    ext: String,
    readonly: bool,
    hidden: bool,
    children: Vec<Node>,
}

fn load_ignore_from_toml(config_path: &Path) -> Vec<String> {
    if !config_path.exists() {
        return Vec::new();
    }

    let text = std::fs::read_to_string(config_path).unwrap();
    let value: toml::Value = toml::from_str(&text).unwrap();

    let mut patterns = Vec::new();

    if let Some(arr) = value
        .get("ignore")
        .and_then(|i| i.get("patterns"))
        .and_then(|p| p.as_array())
    {
        for p in arr {
            if let Some(s) = p.as_str() {
                patterns.push(s.to_string());
            }
        }
    }

    patterns
}

fn should_exclude(full_path: &str, patterns: &Vec<String>) -> bool {

    // println!("CHECK: {}", full_path);    
    
    let normalized = full_path.replace("\\", "/");

    for pat in patterns {
        let p = pat.replace("\\", "/");

        // 1. 完全一致
        if normalized == p {
            return true;
        }

        // 2. ディレクトリ名一致（/target/）
        if normalized.contains(&format!("/{}/", p)) {
            return true;
        }

        // 3. 末尾が target または target/
        if normalized.ends_with(&format!("/{}", p)) || normalized.ends_with(&format!("/{}/", p)) {
            return true;
        }

        // 4. 先頭が target または target/
        if normalized.starts_with(&format!("{}/", p)) || normalized.starts_with(&format!("{}/", p)) {
            return true;
        }

        // 5. “親が除外対象なら子も除外”
        if normalized.contains(&format!("/{}", p)) {
            return true;
        }
    }

    false
}

#[allow(deprecated)]
fn get_modified(metadata: &fs::Metadata) -> String {
    metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| {
            let ts = chrono::NaiveDateTime::from_timestamp_opt(d.as_secs() as i64, 0).unwrap();
            ts.format("%Y-%m-%dT%H:%M:%S").to_string()
        })
        .unwrap_or_default()
}

fn walk(
        path: &Path,
        ignore_patterns: &Vec<String>,
        dirsfirst: bool,
    ) -> Option<Node> {
    let name = path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let mut full_path = fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();

    // Windows の \\?\ プレフィックスを除去
    if full_path.starts_with(r"\\?\") {
        full_path = full_path[4..].to_string();
    }

    // パス区切りを / に統一
    full_path = full_path.replace("\\", "/");


    if should_exclude(&full_path, ignore_patterns) {
        return None;
    }

    let metadata = fs::metadata(path).ok()?;
    let file_type = if metadata.is_dir() { "dir" } else { "file" }.to_string();
    let size = if metadata.is_file() { metadata.len() } else { 0 };
    let modified = get_modified(&metadata);

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    let readonly = metadata.permissions().readonly();

    #[cfg(windows)]
    let hidden = {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x2 != 0
    };

    #[cfg(not(windows))]
    let hidden = false;

    let mut children = Vec::new();

    if metadata.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)
            .ok()?
            .filter_map(|e| e.ok())
            .collect();

        if dirsfirst {
            entries.sort_by(|a, b| {
                let pa = a.path();
                let pb = b.path();

                let ma = pa.is_dir();
                let mb = pb.is_dir();

                match (ma, mb) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => pa.cmp(&pb),
                }
            });
        } else {
            entries.sort_by_key(|e| e.path());
        }

        // entries.sort_by_key(|e| e.path());

        for entry in entries {
            if let Some(child) = walk(&entry.path(), ignore_patterns, dirsfirst) {
                children.push(child);
            }
        }
    }

    Some(Node {
        name,
        path: path.to_string_lossy().to_string(),
        file_type,
        size,
        modified,
        ext,
        readonly,
        hidden,
        children,
    })
}
//
// TEXT OUTPUT
//
fn print_text(node: &Node, prefix: &str, is_last: bool, show_prop: bool) {
    let connector = if is_last { "└── " } else { "├── " };

    if show_prop {
        println!(
            "{}{}{} <{}, ext={}, size={}, modified={}, readonly={}, hidden={}>",
            prefix,
            connector,
            node.name,
            node.ext,
            node.file_type,
            node.size,
            node.modified,
            node.readonly,
            node.hidden
        );
    } else {
        println!("{}{}{}", prefix, connector, node.name);
    }

    let new_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    let last_index = node.children.len().saturating_sub(1);

    for (i, child) in node.children.iter().enumerate() {
        print_text(child, &new_prefix, i == last_index, show_prop);
    }
}

//
// CSV OUTPUT
//
fn print_csv(node: &Node) {
    println!(
        "{},{},{},{},{},{},{},{},{}",
        node.path,
        node.name,
        node.file_type,
        node.size,
        node.modified,
        node.ext,
        node.readonly,
        node.hidden,
        node.children.len()
    );

    for child in &node.children {
        print_csv(child);
    }
}

fn print_help() {
    println!("ixtree - directory tree viewer");
    println!();
    println!("Usage:");
    println!("  ixtree [PATH] [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -p, --property          Show file properties");
    println!("  -I, --ignore PATTERN    Add ignore pattern");
    println!("      --dirsfirst         Show directories first");
    println!("      --format text|csv|json");
    println!("      --config FILE       Specify config file");
    println!("  -h, --help              Show this help");
    println!("  -v, --version           Show version");
}

fn print_version() {
    println!("ixtree {}", env!("CARGO_PKG_VERSION"));
}

#[warn(unused_variables)]
fn main() {

    let args: Vec<String> = env::args().collect();


    // -----------------------------
    // help / version は最優先で処理
    // -----------------------------
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--version" | "-v" => {
                print_version();
                return;
            }
            _ => {}
        }
    }
    // -----------------------------
    // 1. デフォルト値
    // -----------------------------
    let mut target_path = ".".to_string();
    let mut config_path: Option<PathBuf> = None;
    let mut exclude_patterns: Vec<String> = Vec::new();
    let mut show_prop = false;
    let mut format = "text";
    let mut dirsfirst = false;

    // -----------------------------
    // 2. 引数パース
    // -----------------------------
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dirsfirst" => {
                dirsfirst = true;
                i += 1;
            }

            "-p" | "--property" => {
                show_prop = true;
                i += 1;
            }

            "-I" | "--ignore" => {
                exclude_patterns.push(args[i + 1].clone());
                i += 2;
            }

            "--format" => {
                format = &args[i + 1];
                i += 2;
            }

            "--config" => {
                config_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }

            // ★ 最初に出てきた非オプション引数を探索対象にする
            p => {
                target_path = p.to_string();
                i += 1;
            }
        }
    }

    // -----------------------------
    // 3. 設定ファイルの読み込み
    // -----------------------------
    let root_path = env::current_dir().unwrap();

    // 明示的に --config が指定された場合
    let config_file = if let Some(cfg) = config_path {
        cfg
    } else {
        root_path.join(".ixtree.toml")
    };

    // 設定ファイルが無ければ警告
    if !config_file.exists() {
        eprintln!(
            "warning: config file not found: {}",
            config_file.display()
        );
    }

    // TOML の ignore を読み込む
    let mut ignore_patterns = load_ignore_from_toml(&config_file);

    // -I の追加パターン
    ignore_patterns.extend(exclude_patterns);

    // -----------------------------
    // 4. 探索対象パスを PathBuf に変換
    // -----------------------------
    let target_path = PathBuf::from(target_path);

    // -----------------------------
    // 5. walk 実行
    // -----------------------------
    let root = walk(
        &target_path,
        &ignore_patterns,
        dirsfirst
    ).unwrap();

    let abs = root_path.canonicalize().unwrap();

    match format {
        "text" => {
            let last_index = root.children.len().saturating_sub(1);
            let mut s = abs.to_string_lossy().to_string();
            // Windows の \\?\ プレフィックスを除去
            if s.starts_with(r"\\?\") {
                s = s[4..].to_string();
            }
            println!("{}", s);            
            for (i, child) in root.children.iter().enumerate() {
                print_text(child, "", i == last_index, show_prop);
            }
        }
        "csv" => {
            println!("path,name,type,size,modified,ext,readonly,hidden,child_count");
            for child in &root.children {
                print_csv(child);
            }
        }
        "json" => {
            let s = serde_json::to_string_pretty(&root.children).unwrap();
            println!("{}", s);
        }
        _ => {}
    }
}
