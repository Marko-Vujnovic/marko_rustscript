pub mod common_dirs; pub use common_dirs::*;
pub mod project; pub use project::*;

fn is_newer_than<P: AsRef<std::path::Path>>(l: P, r: &std::path::Path) -> core::result::Result<bool, std::io::Error> {
    Ok(std::fs::metadata(l)?.modified()? > std::fs::metadata(r)?.modified()?)
}

fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for p in path.split(":") {
            let p_str = format!("{}/{}", p, program);
            if std::fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

async fn installed_rust_is_too_old(bf: &std::path::Path) -> core::result::Result<bool, std::io::Error> {
    Ok(!async_process::Command::new("cargo").arg("check").arg("--release").arg("--all-targets").arg("--tests").arg("--target-dir").arg(&bf).output().await?.status.success())
}

pub fn copy(from: &std::path::Path, to: &std::path::Path) -> core::result::Result<(), std::io::Error> {
    let read_from = std::fs::File::open(&from)?;
    let mut reader = std::io::BufReader::new(&read_from);
    let append_to = std::fs::OpenOptions::new().append(true).create(true).open(&to)?;
    let mut writer = std::io::BufWriter::new(&append_to);
    let mut length = 1;
    // while let Ok(n) = read_from.read(&mut buf[..]) {
    while length > 0 {
        let buffer = std::io::BufRead::fill_buf(&mut reader).unwrap();
        std::io::Write::write(&mut writer, &buffer)?;
        length = buffer.len();
        std::io::BufRead::consume(&mut reader, length);
    }
    // std::fs::set_permissions(&to, <std::fs::Permissions as std::os::unix::prelude::PermissionsExt>::from_mode(0o755))?;
    Ok(())
}

pub async fn cargo_build(script_project_folder: &std::path::Path) -> core::result::Result<(), std::io::Error> {
    let proj_name = script_project_folder.file_stem().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Couldn't determine project name"))?.to_str().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "&OsStr -> &str failed"))?;
    let toml_file = script_project_folder.join("Cargo.toml");
    let bf = get_the_BuildFolders_folder().join(&proj_name);
    let exe = bf.join(format!("release/{}", &proj_name));
    let no_need_to_build = exe.exists() && is_newer_than(&exe, &toml_file)?;
    if no_need_to_build == false {
        std::env::set_current_dir(&script_project_folder)?;
        std::env::set_var("CARGO_NET_GIT_FETCH_WITH_CLI", "true");
        let rust_is_installed = is_program_in_path("cargo");
        if (!rust_is_installed || installed_rust_is_too_old(&bf).await?) && std::env::consts::OS == "linux" {
            let env_descr = envie::Environment{
                name: "Rust dev env with a very recent rustc version".to_string(),
                installed_packages: vec![
                    "bash".to_string(), "patchelf".to_string(),
                    "nixpkgs_ToUseRustFrom.rustc".to_string(),
                    "nixpkgs_ToUseRustFrom.cargo".to_string(),
                ],
                depends_on: vec![],
                inherits_from: vec![],
            };
            let mut env_shell = envie::get_shell(&env_descr).await?;
            std::io::Write::write_all(&mut env_shell.stdin.as_mut().unwrap(), format!("CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build --target-dir {:?} --release && patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 {}\n", &bf, exe.to_str().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "exe path to_str() failed"))?).as_bytes())?;
            let cmd_result = env_shell.wait_with_output()?;
            // println!("cmd_result: {:?}", &cmd_result);
        }
        else {
            let cmd_process = std::process::Command::new("cargo").arg("build").arg("--release").arg("--target-dir").arg(&bf).spawn()?; cmd_process.wait_with_output()?;
        }
    }
    Ok(())
}

pub async fn main_(script_path: &str, tui_is_occupying_stdout: bool) -> core::result::Result<(), std::io::Error> {
    unsafe{ envie::SANDBOX_TO_USE = envie::SandboxRuntime::Proot; }
    let script_p: std::path::PathBuf = script_path.into(); let script_p = std::fs::canonicalize(script_p)?;
    let script_name: String = script_p.file_stem().unwrap().to_string_lossy().to_string();
    let script_project = Project{ name: script_name };

    if !script_p.exists() { panic!("No such file: {:?}", &script_path); }

    let script_project_folder = get_the_script_projects_folder().join(&script_project.name);
    let project_f_is_stale = !script_project_folder.exists() || is_newer_than(&script_p, &script_project_folder.join("Cargo.toml"))?;
    if project_f_is_stale {
        // if !script_project_folder.exists() { std::fs::create_dir_all(&script_project_folder)?; }
        // println_!("Project folder is stale");
        marko_plaintext_archive::unpack2(&script_path, &get_the_script_projects_folder())?;
    }
    
    let bf = project_get_the_scripts_build_folder(&script_project);
    let name = &script_project.name; let exe = bf.join(format!("release/{}", &name));

    cargo_build(&script_project_folder).await?;
    // println!("exe: {:?}", &exe);
    // println!("exe unquoted: {}", exe.to_string_lossy());
    if !tui_is_occupying_stdout {
        let cmd_proc = std::process::Command::new(&exe).stdout(std::process::Stdio::inherit()).spawn()?; cmd_proc.wait_with_output()?;
    }
    else {
        let cmd_proc = std::process::Command::new("xterm").args(&["-e", "/bin/bash", "-i", "-c", &format!("{};{}", exe.to_str().unwrap(), "sleep 5")]).stdout(std::process::Stdio::inherit()).spawn()?; cmd_proc.wait_with_output()?;
    }
    Ok(())
}

pub fn parse_args(mut args: impl Iterator<Item = String>) -> (std::collections::HashMap<String, String>, Vec<String>) {
    let mut flags = std::collections::HashMap::new();
    let mut positionals = Vec::new();
    while let Some(arg) = args.next() {
        if let Some(flag) = arg.strip_prefix("-") {
            if let Some(option) = flag.strip_prefix("-") {
                flags.insert(option.into(), args.next().unwrap_or_default());
            } else {
                for c in flag.chars() {
                    flags.insert(c.into(), String::from("true"));
                }
            }
        } else {
            positionals.push(arg);
        }
    }
    (flags, positionals)
}

pub fn os_username() -> String { uid_to_username(geteuid_()).unwrap() }

pub fn uid_to_username(uid: u32) -> core::option::Option<String> { unsafe {
    let mut result = std::ptr::null_mut();
    let amt = match libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) {
        n if n < 0 => 512 as usize,
        n => n as usize,
    };
    let mut buf = Vec::with_capacity(amt);
    let mut passwd: libc::passwd = std::mem::zeroed();

    match libc::getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(),
                           buf.capacity() as libc::size_t,
                           &mut result) {
        0 if !result.is_null() => {
            let ptr = passwd.pw_name as *const _;
            let username = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
            Some(username)
        },
        _ => None
    }
}}

pub struct ProgramInfo { name: &'static str }
static program_info: ProgramInfo = ProgramInfo {
    name: "rustscript",
};

#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
    fn getegid() -> u32;
}

fn geteuid_() -> u32 { unsafe { geteuid() } }

#[macro_export]
macro_rules! println_ {
    ($($arg:expr),*) => {
        $(print!("{}", $arg);)*
        println!();
    };
}

#[macro_export]
macro_rules! print_ {
    ($($arg:expr),*) => {
        $(print!("{}", $arg);)*
    };
}