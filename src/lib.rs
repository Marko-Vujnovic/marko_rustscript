pub mod common_dirs; pub use common_dirs::*;
pub mod project; pub use project::*;

pub async fn main_(script_path: &str) -> core::result::Result<(), std::io::Error> {
    // println!("{}", script_path);
    println_!(script_path);

    marko_plaintext_archive::unpack2(&script_path, &get_the_script_projects_folder());

    let scriptProject = Project{ name: "example".to_string() };
    std::env::set_current_dir(&project_get_the_script_folder(&scriptProject))?;
    std::env::set_var("CARGO_NET_GIT_FETCH_WITH_CLI", "true");
    let bf = project_get_the_scripts_build_folder(&scriptProject);
    let out = async_process::Command::new("cargo").arg("build").arg("--release").arg("--target-dir").arg(&bf).output().await?;
    println!("cargo: {}", std::str::from_utf8(&out.stdout).unwrap());
    println!("cargo: {}", std::str::from_utf8(&out.stderr).unwrap());

    let name = &scriptProject.name; let exe = bf.join(format!("release/{name}"));

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