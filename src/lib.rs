pub mod common_dirs; pub use common_dirs::*;
pub mod project; pub use project::*;

fn is_newer_than<P: AsRef<std::path::Path>>(l: P, r: &std::path::Path) -> core::result::Result<bool, std::io::Error> {
    Ok(std::fs::metadata(l)?.modified()? > std::fs::metadata(r)?.modified()?)
}

pub async fn main_(script_path: &str) -> core::result::Result<(), std::io::Error> {
    let script_p: std::path::PathBuf = script_path.into(); let script_p = std::fs::canonicalize(script_p)?;
    let script_name: String = script_p.file_stem().unwrap().to_string_lossy().to_string();
    let scriptProject = Project{ name: script_name };

    if !script_p.exists() { panic!("No such file: {:?}", &script_path); }

    marko_plaintext_archive::unpack2(&script_path, &get_the_script_projects_folder());

    let script_project_folder = get_the_script_projects_folder().join(&scriptProject.name);
    std::env::set_current_dir(&script_project_folder)?;
    std::env::set_var("CARGO_NET_GIT_FETCH_WITH_CLI", "true");
    let bf = project_get_the_scripts_build_folder(&scriptProject);
    let name = &scriptProject.name; let exe = bf.join(format!("release/{name}"));
    let no_need_to_build = exe.exists() && is_newer_than(&exe, &script_p)?;
    if no_need_to_build == false {
        let out = async_process::Command::new("cargo").arg("build").arg("--release").arg("--target-dir").arg(&bf).output().await?;
        println!("cargo: {}", std::str::from_utf8(&out.stdout).unwrap());
        println!("cargo: {}", std::str::from_utf8(&out.stderr).unwrap());
    }

    let out = async_process::Command::new(&exe).output().await?; println!("{}", std::str::from_utf8(&out.stdout).unwrap());
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