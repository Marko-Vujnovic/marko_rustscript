use crate as rustscript; use self::rustscript::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Project { pub name:String }

pub fn project_get_the_script_folder(project: &Project) -> std::path::PathBuf { get_the_script_projects_folder().join(&project.name) }

pub fn project_get_the_scripts_build_folder(project: &Project) -> std::path::PathBuf { get_the_BuildFolders_folder().join(&project.name) }