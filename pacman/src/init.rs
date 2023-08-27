use crate::args::ProjectKind;
use std::path::Path;

const MAIN: &str = include_str!("..\\templates\\init\\main.rd");
const LIB: &str = include_str!("..\\templates\\init\\lib.rd");
const CONFIG: &str = include_str!("..\\templates\\init\\Ruda.toml");
const GITIGNORE: &str = include_str!("..\\templates\\init\\.gitignore");

pub fn init(
    path: &str,
    kind: ProjectKind,
    name: Option<String>,
    version: Option<String>,
    author: Option<String>,
) {
    // init project
    let path = Path::new(path);
    let name = name.unwrap_or("project".to_string());
    let version = version.unwrap_or("0.1.0".to_string());
    let author = author.unwrap_or("author".to_string());
    let kind = match kind {
        ProjectKind::Lib => "lib",
        ProjectKind::Bin => "bin",
    };
    let config = CONFIG
        .replace("{{name}}", &name)
        .replace("{{version}}", &version)
        .replace("{{kind}}", kind)
        .replace("{{author}}", &author);
    let main = match kind {
        "lib" => LIB,
        "bin" => MAIN,
        _ => unreachable!("Invalid project kind"),
    };

    // check if path exists
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }

    // check if path is empty
    if let Ok(entries) = std::fs::read_dir(path) {
        if entries.count() > 0 {
            println!("Directory is not empty. Continue? [y/N]");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if input.trim().to_lowercase() != "y" {
                return;
            }
        }
    }

    // create files
    // src folder
    let src_path = path.join("src");
    std::fs::create_dir_all(&src_path).unwrap();
    // main.rd
    let main_path = src_path.join("main.rd");
    std::fs::write(&main_path, main).unwrap();
    // Ruda.toml
    let config_path = path.join("Ruda.toml");
    std::fs::write(&config_path, config).unwrap();
    // .gitignore
    let gitignore_path = path.join(".gitignore");
    std::fs::write(&gitignore_path, GITIGNORE).unwrap();

    // print success message
    println!("Successfully created project at {}", path.display());
}
