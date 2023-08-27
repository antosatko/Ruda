use git2::Repository;
use git_url_parse::GitUrl;

/// Install a package from a remote repository
/// returns the path to the package
pub fn install(url: &str, version: &str) -> String {
    println!("installing {}@{}", url, version);
    let ruda_path = std::env::var("RUDA_PATH").unwrap();
    let packages_path = std::path::Path::new(&ruda_path).join("packages");
    // create packages directory if it doesn't exist
    if !packages_path.exists() {
        std::fs::create_dir_all(&packages_path).unwrap();
    }
    // parse git url
    let git_url = GitUrl::parse(url).unwrap();
    // create owner directory if it has an owner
    let author_path = match git_url.owner {
        Some(owner) => {
            let owner_path = packages_path.join(owner);
            if !owner_path.exists() {
                std::fs::create_dir_all(&owner_path).unwrap();
            }
            owner_path
        },
        None => packages_path,
    };
    // create package directory
    let package_path = author_path.join(git_url.name);
    if !package_path.exists() {
        std::fs::create_dir_all(&package_path).unwrap();
    }else {
        // remove package directory if it exists
        std::fs::remove_dir_all(&package_path).unwrap();
        // create package directory
        std::fs::create_dir_all(&package_path).unwrap();
    }
    // download package
    let _repo = match Repository::clone(url, &package_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };

    let path = package_path.to_str().unwrap().to_string();
    path
}

/// Uninstall a package
pub fn uninstall(url: &str, version: &str) {
    let ruda_path = std::env::var("RUDA_PATH").unwrap();
    let packages_path = std::path::Path::new(&ruda_path).join("packages");
    // parse git url
    let git_url = GitUrl::parse(url).unwrap();
    // get owner path
    let author_path = match git_url.owner {
        Some(owner) => packages_path.join(owner),
        None => packages_path,
    };
    // get package path
    let package_path = author_path.join(git_url.name);
    // remove package directory if it exists
    if package_path.exists() {
        println!("removing {}", package_path.display());
        std::fs::remove_dir_all(&package_path).unwrap();
    }else {
        println!("package {} not found", package_path.display());
    }
}


/// returns path to package from provided url
/// if it is not url, it returns the provided url
pub fn path(url: &str, version: &str) -> String {
    let ruda_path = std::env::var("RUDA_PATH").unwrap();
    let packages_path = std::path::Path::new(&ruda_path).join("packages");
    // if url is not a git url, return url
    if !is_remote(url) {
        return url.to_string();
    }
    // parse git url
    let git_url = GitUrl::parse(url).unwrap();
    // get owner path
    let author_path = match git_url.owner {
        Some(owner) => packages_path.join(owner),
        None => packages_path,
    };
    // get package path
    let package_path = author_path.join(git_url.name);
    // return package path
    package_path.to_str().unwrap().to_string()
}

/// returns true if package is installed
pub fn is_installed(url: &str, version: &str) -> bool {
    let ruda_path = std::env::var("RUDA_PATH").unwrap();
    let packages_path = std::path::Path::new(&ruda_path).join("packages");
    // parse git url
    let git_url = GitUrl::parse(url).unwrap();
    // get owner path
    let author_path = match git_url.owner {
        Some(owner) => packages_path.join(owner),
        None => packages_path,
    };
    // get package path
    let package_path = author_path.join(git_url.name);
    // return package path
    package_path.exists()
}

pub fn is_remote(url: &str) -> bool {
    url.starts_with("git") || url.ends_with(".git") || url.contains("://")
}