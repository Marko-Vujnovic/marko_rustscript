use crate as rustscript; use self::rustscript::*;

pub fn get_app_folder() -> std::path::PathBuf { let mut dir = std::path::PathBuf::new(); let username = os_username(); dir.push(format!("/home/{}", &username)); dir.join(".cache").join(&program_info.name) }
pub fn cwd() -> std::path::PathBuf { std::env::current_dir().unwrap() }

pub fn get_the_script_projects_folder() -> std::path::PathBuf { get_app_folder().join("ScriptProjects") }
pub fn get_the_BuildFolders_folder() -> std::path::PathBuf { get_app_folder().join("BuildFolders") }