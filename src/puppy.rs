use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

pub fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| help_message())?;

    match command.as_str() {
        "install" => {
            let repo_arg = args.next().ok_or_else(|| "Missing repository argument. Usage: puppy install <owner/repo | github url>".to_string())?;
            if args.next().is_some() {
                return Err("Too many arguments. Usage: puppy install <owner/repo | github url>".to_string());
            }
            install_command(&repo_arg)
        }
        "list" => list_command(),
        "about" | "info" => about_command(),
        "uninstall" => {
            let repo_arg = args.next().ok_or_else(|| "Missing repository argument. Usage: puppy uninstall <owner/repo | github url>".to_string())?;
            if args.next().is_some() {
                return Err("Too many arguments. Usage: puppy uninstall <owner/repo | github url>".to_string());
            }
            uninstall_command(&repo_arg)
        }
        "help" | "--help" | "-h" => {
            println!("{}", help_message());
            Ok(())
        }
        _ => Err(format!("Unknown command: '{}'. Try 'puppy help' for a list of commands.", command)),
    }
}

fn help_message() -> String {
    "puppy - install GitHub projects locally\n\nCommands:\n  puppy install <owner/repo | github url>   Install a GitHub project locally\n  puppy list                               List installed packages\n  puppy uninstall <owner/repo | github url> Remove installed package and source\n  puppy about                              Show information about puppy\n  puppy help                               Show this help message\n\nExamples:\n  puppy install sharkdp/bat\n  puppy list\n  puppy uninstall sharkdp/bat\n  puppy about\n\nInstalled sources are stored under ~/.puppy/src and binaries under ~/.puppy/bin\n".to_string()
}

fn install_command(source: &str) -> Result<(), String> {
    let spec = parse_repo_spec(source)?;
    let home = home_dir()?;
    let puppy_root = home.join(".puppy");
    let src_root = puppy_root.join("src");
    let bin_root = puppy_root.join("bin");

    create_dir(&puppy_root)?;
    create_dir(&src_root)?;
    create_dir(&bin_root)?;

    let target_dir = src_root.join(format!("{}-{}", spec.owner, spec.repo));
    clone_repo(&spec, &target_dir)?;

    let project = detect_project_type(&target_dir);
    let result = match project {
        ProjectType::Cargo => install_cargo(&target_dir, &puppy_root),
        ProjectType::Go => install_go(&target_dir, &bin_root),
        ProjectType::CMake => install_cmake(&target_dir, &puppy_root),
        ProjectType::Make => install_make(&target_dir, &puppy_root),
        ProjectType::Node => install_node(&target_dir, &puppy_root),
        ProjectType::Unknown => install_fallback(&target_dir),
    };

    if result.is_ok() {
        info("Done. Add ~/.puppy/bin to your PATH if it is not already available.");
    }
    result
}

fn uninstall_command(source: &str) -> Result<(), String> {
    let spec = parse_repo_spec(source)?;
    let home = home_dir()?;
    let src_root = home.join(".puppy").join("src");
    let bin_root = home.join(".puppy").join("bin");
    let target_dir = src_root.join(format!("{}-{}", spec.owner, spec.repo));
    let binary_path = bin_root.join(&spec.repo);

    if target_dir.exists() {
        info(&format!("Removing source directory {}", target_dir.display()));
        fs::remove_dir_all(&target_dir).map_err(|err| format!("Failed to remove {}: {}", target_dir.display(), err))?;
    } else {
        info(&format!("Source directory not found: {}", target_dir.display()));
    }

    if binary_path.exists() {
        info(&format!("Removing binary {}", binary_path.display()));
        fs::remove_file(&binary_path).map_err(|err| format!("Failed to remove {}: {}", binary_path.display(), err))?;
    } else {
        info(&format!("Binary not found: {}", binary_path.display()));
    }

    info("Uninstall complete.");
    Ok(())
}

fn list_command() -> Result<(), String> {
    let home = home_dir()?;
    let src_root = home.join(".puppy").join("src");
    if !src_root.exists() {
        info("No packages installed yet.");
        return Ok(());
    }

    let mut entries = fs::read_dir(&src_root)
        .map_err(|err| format!("Could not read {}: {}", src_root.display(), err))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    entries.sort();

    if entries.is_empty() {
        info("No packages installed yet.");
    } else {
        info("Installed packages:");
        for name in entries {
            println!("  {}", name);
        }
    }
    Ok(())
}

fn about_command() -> Result<(), String> {
    println!("puppy - GitHub package installer\n");
    println!("  puppy installs GitHub repositories, detects the build system, and installs binaries into ~/.puppy/bin");
    println!("  It stores source repositories under ~/.puppy/src.");
    println!("\nHow it works:");
    println!("  1. Clone the repository from GitHub into ~/.puppy/src");
    println!("  2. Detect the project type by checking for Cargo.toml, go.mod, CMakeLists.txt, Makefile, or package.json");
    println!("  3. Run the appropriate build/install commands for the detected project type");
    println!("  4. Place executables into ~/.puppy/bin for easy use");
    println!("\nSupported project types:");
    println!("  - Rust (Cargo)");
    println!("  - Go");
    println!("  - CMake");
    println!("  - Make");
    println!("  - Node/npm");
    println!("\nOwner:");
    println!("  puppy is owned and built by the current project maintainer");
    println!("\nPuppy logo if i was a dog:");
    println!(r"   _   _   _   _   _   _  ");
    println!(r"  / \ / \ / \ / \ / \ / \ ");
    println!(r" ( p | u | p | p | y | ! )");
    println!(r"  \_/ \_/ \_/ \_/ \_/ \_/ ");
    println!("\nUse 'puppy help' to list commands.");
    Ok(())
}

struct RepoSpec {
    owner: String,
    repo: String,
    url: String,
}

fn parse_repo_spec(input: &str) -> Result<RepoSpec, String> {
    if input.starts_with("http://")
        || input.starts_with("https://")
        || input.starts_with("git@")
        || input.starts_with("ssh://")
    {
        parse_git_url(input)
    } else {
        parse_owner_repo(input)
    }
}

fn parse_owner_repo(input: &str) -> Result<RepoSpec, String> {
    let mut parts = input.trim_end_matches(|c| c == '/' || c == '.')
        .trim_end_matches(".git")
        .split('/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(owner), Some(repo), None) if !owner.is_empty() && !repo.is_empty() => {
            let repo_name = repo.trim_end_matches(".git");
            Ok(RepoSpec {
                owner: owner.to_string(),
                repo: repo_name.to_string(),
                url: format!("https://github.com/{}/{}.git", owner, repo_name),
            })
        }
        _ => Err(format!("Could not parse repository spec: '{}'. Use owner/repo or GitHub URL.", input)),
    }
}

fn parse_git_url(input: &str) -> Result<RepoSpec, String> {
    let trimmed = if input.starts_with("git@") {
        input
    } else {
        input.trim_end_matches('/').trim_end_matches(".git")
    };

    if let Some(stripped) = trimmed.strip_prefix("git@github.com:") {
        parse_owner_repo(stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("ssh://git@github.com/") {
        parse_owner_repo(stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("https://github.com/") {
        parse_owner_repo(stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("http://github.com/") {
        parse_owner_repo(stripped)
    } else {
        Err(format!("Unsupported GitHub URL: '{}'.", input))
    }
}

fn home_dir() -> Result<PathBuf, String> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME environment variable is not set.".to_string())
}

fn create_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|err| format!("Could not create {}: {}", path.display(), err))
}

fn clone_repo(spec: &RepoSpec, dest: &Path) -> Result<(), String> {
    if dest.exists() {
        info(&format!("Repository already exists at {}. Skipping clone.", dest.display()));
        Ok(())
    } else {
        info(&format!("Cloning {} into {}", spec.url, dest.display()));
        run_command(
            "git",
            &["clone", "--depth", "1", &spec.url, dest.to_str().ok_or("Invalid destination path")?],
            None,
        )
    }
}

#[derive(Debug, Copy, Clone)]
enum ProjectType {
    Cargo,
    Go,
    CMake,
    Make,
    Node,
    Unknown,
}

fn detect_project_type(repo_dir: &Path) -> ProjectType {
    if repo_dir.join("Cargo.toml").is_file() {
        ProjectType::Cargo
    } else if repo_dir.join("go.mod").is_file() {
        ProjectType::Go
    } else if repo_dir.join("CMakeLists.txt").is_file() {
        ProjectType::CMake
    } else if repo_dir.join("Makefile").is_file() || repo_dir.join("makefile").is_file() {
        ProjectType::Make
    } else if repo_dir.join("package.json").is_file() {
        ProjectType::Node
    } else {
        ProjectType::Unknown
    }
}

fn install_cargo(repo_dir: &Path, install_root: &Path) -> Result<(), String> {
    info("Detected Cargo project.");
    let prefix = install_root.to_str().ok_or("Invalid install root path.")?;
    run_command(
        "cargo",
        &["install", "--path", ".", "--root", prefix],
        Some(repo_dir),
    )?;
    info(&format!("Installed Cargo package into {}", install_root.join("bin").display()));
    Ok(())
}

fn install_go(repo_dir: &Path, bin_dir: &Path) -> Result<(), String> {
    info("Detected Go project.");
    let repo_name = repo_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("go-app");

    let output_path = bin_dir.join(repo_name);
    run_command(
        "go",
        &["build", "-o", output_path.to_str().ok_or("Invalid output path")?],
        Some(repo_dir),
    )?;
    set_executable(&output_path)?;
    info(&format!("Installed Go binary to {}", output_path.display()));
    Ok(())
}

fn install_cmake(repo_dir: &Path, install_root: &Path) -> Result<(), String> {
    info("Detected CMake project.");
    let build_dir = repo_dir.join("build-puppy");
    let prefix = install_root.to_str().ok_or("Invalid install root path.")?;

    run_command(
        "cmake",
        &["-S", ".", "-B", build_dir.to_str().ok_or("Invalid build directory")?, &format!("-DCMAKE_INSTALL_PREFIX={}", prefix), "-DCMAKE_BUILD_TYPE=Release"],
        Some(repo_dir),
    )?;
    run_command(
        "cmake",
        &["--build", build_dir.to_str().ok_or("Invalid build directory")?, "--target", "install", "--config", "Release"],
        None,
    )?;
    info(&format!("Installed CMake project into {}", install_root.display()));
    Ok(())
}

fn install_make(repo_dir: &Path, install_root: &Path) -> Result<(), String> {
    info("Detected Makefile project.");
    run_command("make", &[], Some(repo_dir))?;
    let prefix = install_root.to_str().ok_or("Invalid install root path.")?;
    match run_command("make", &["install", &format!("PREFIX={}", prefix)], Some(repo_dir)) {
        Ok(()) => {
            info(&format!("Installed Makefile project into {}", install_root.display()));
            Ok(())
        }
        Err(err) => {
            info("Build succeeded, but automatic install failed.");
            info("If the project provides an install target, try running: make install PREFIX=~/.puppy");
            Err(err)
        }
    }
}

fn install_node(repo_dir: &Path, install_root: &Path) -> Result<(), String> {
    info("Detected Node project.");
    let prefix = install_root.to_str().ok_or("Invalid install root path.")?;
    match run_command("npm", &["install", "--prefix", prefix], Some(repo_dir)) {
        Ok(()) => {
            info(&format!("Installed Node package into {}", install_root.display()));
            info("Add ~/.puppy/bin and ~/.puppy/lib/node_modules/.bin to your PATH if needed.");
            Ok(())
        }
        Err(err) => {
            info("Node install failed. Ensure npm is installed and package.json is valid.");
            Err(err)
        }
    }
}

fn install_fallback(repo_dir: &Path) -> Result<(), String> {
    info("Could not detect a supported build system.");
    info(&format!("Repository cloned to {}", repo_dir.display()));
    info("You can inspect the repository and build it manually.");
    info("Common commands: cargo build --release, go build, cmake/make, npm install.");
    Ok(())
}

fn set_executable(path: &Path) -> Result<(), String> {
    let mut perms = fs::metadata(path)
        .map_err(|err| format!("Failed to read metadata for {}: {}", path.display(), err))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
        .map_err(|err| format!("Failed to set executable permissions for {}: {}", path.display(), err))
}

fn run_command(cmd: &str, args: &[&str], current_dir: Option<&Path>) -> Result<(), String> {
    debug(&format!("Running command: {} {}", cmd, args.join(" ")));
    let mut command = Command::new(cmd);
    command.args(args);
    if let Some(dir) = current_dir {
        command.current_dir(dir);
    }
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let status = command
        .status()
        .map_err(|err| format!("Failed to execute {}: {}", cmd, err))?;
    if status.success() {
        Ok(())
    } else {
        Err(status_to_string(status, cmd))
    }
}

fn status_to_string(status: ExitStatus, cmd: &str) -> String {
    match status.code() {
        Some(code) => format!("Command '{}' failed with exit code {}", cmd, code),
        None => format!("Command '{}' failed by signal", cmd),
    }
}

fn info(message: &str) {
    println!("[puppy] {}", message);
}

fn debug(message: &str) {
    if env::var_os("PUPPY_DEBUG").is_some() {
        eprintln!("[puppy debug] {}", message);
    }
}
