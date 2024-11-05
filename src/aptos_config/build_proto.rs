use log::{error, info, LevelFilter};
use std::fs::{create_dir_all, read_dir, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The relative proto directory path from the cargo.toml
pub const RELATIVE_PROTO_DIR_PATH: &str = "src/aptos_config/proto_src";
/// The relative proto output directory path from the cargo.toml
pub const RELATIVE_PROTO_OUT_DIR_PATH: &str = "src/aptos_config/proto_codegen";

/// Goes through a directory and all of its subdirectories
/// and returns a vector of PathBufs pointing to all
/// Proto files.
fn collect_proto_files(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    // Stores all the proto files
    let mut proto_files: Vec<PathBuf> = Vec::new();
    // Retrieve the directory's entries
    let entries = match read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            error!("[aptos-etl] Current Dir: {:?}", std::env::current_dir());
            error!(
                "[aptos-etl] Failed to read from directory `{:?}` due to {}",
                dir, error
            );
            return Err(Box::new(error));
        }
    };
    // Go through the entries and gather protos
    for entry in entries {
        // Attempt to read current direntry
        match entry {
            Ok(dir_entry) => {
                // Get path of the current entry
                let path = dir_entry.path();
                // If it is a directory, start the recursive
                // process
                if path.is_dir() {
                    // Attempt to collect from subdirectory
                    match collect_proto_files(&path) {
                        Ok(protos) => proto_files.extend(protos),
                        Err(err) => return Err(err),
                    }
                } else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("proto") {
                    // If the file is a .proto file, add it to the list.
                    proto_files.push(path);
                }
            }
            Err(error) => {
                error!("Failed to read entry due to {}", error);
                return Err(Box::new(error));
            }
        }
    }

    Ok(proto_files)
}

fn collect_output_rs_files(
    dir: &Path,
    ignore_mod: bool,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let entries = match read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            error!("Failed to read from directory `{:?}` due to {}", dir, error);
            return Err(Box::new(error));
        }
    };

    let mut rustfiles = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                error!("[aptos-etl] Folder: {:?}", std::env::current_dir());
                error!("[aptos-etl] Failed to get file: {:?}", error);
                return Err(Box::new(error));
            }
        };
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if ignore_mod {
                if path.file_stem().and_then(|s| s.to_str()) != Some("mod") {
                    rustfiles.push(path);
                }
            } else {
                rustfiles.push(path);
            }
        }
    }

    Ok(rustfiles)
}

fn parse_outfile(path: &PathBuf) -> Vec<String> {
    match path.file_stem() {
        Some(filestem) => match filestem.to_str() {
            Some(filestem) => filestem.split('.').map(String::from).collect(),
            None => panic!("Failed to change filestem to string"),
        },
        None => panic!("Failed to parse outfile: {:?}", path),
    }
}

/// Create the mod.rs file
fn create_mod(outfiles: Vec<PathBuf>, modfile: File) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Debug)]
    struct Node {
        pub name: String,
        pub path: Option<PathBuf>,
        pub children: std::collections::HashMap<String, Node>,
        pub is_root: bool,
    }

    impl Node {
        pub fn fill_path(&mut self, new_path: &Path) {
            match &self.path {
                Some(current_path) if current_path.clone() != new_path => {
                    panic!("Path already set to {:?}", current_path)
                }
                Some(_) => (),
                None => self.path = Some(new_path.to_path_buf()),
            }
        }

        pub fn new(name: &str, path: Option<&Path>, is_root: bool) -> Self {
            Self {
                name: name.to_string(),
                path: path.map(|path| path.to_path_buf()),
                children: std::collections::HashMap::new(),
                is_root,
            }
        }

        pub fn add(&mut self, name: &str, path: Option<&Path>) -> &mut Node {
            let newnode = self
                .children
                .entry(name.to_string())
                .or_insert(Node::new(name, path, false));

            if let Some(path) = path {
                newnode.fill_path(path);
            }

            newnode
        }

        pub fn add_file(&mut self, path: &PathBuf) {
            // Parse it
            let mut parsed = parse_outfile(path);
            let final_part = match parsed.pop() {
                Some(final_part) => final_part,
                None => {
                    error!("Failed to extract final part");
                    panic!("Failed to extract final part");
                }
            };

            // Add all intermediate things
            let mut curnode = self;
            for part in parsed.iter() {
                curnode = curnode.add(part, None);
            }

            // Add the final part with the filepath
            curnode.add(&final_part, Some(path));
        }

        fn r_to_modfile<T: Write>(&self, f: &mut BufWriter<T>) -> std::io::Result<()> {
            if !self.is_root {
                writeln!(f, "pub mod {} {{", self.name)?;
                // If it is for a file, go ahead and add it in
                if let Some(path) = self.path.clone() {
                    writeln!(
                        f,
                        "include!({:?});",
                        path.file_name().expect("Missing filename")
                    )?;
                }
            }

            // Go through and add any submodule children
            for subnode in self.children.values() {
                subnode.r_to_modfile(f)?;
            }

            if !self.is_root {
                // Add in terminating }
                writeln!(f, "}}")?;
            }

            // Return Ok
            Ok(())
        }

        pub fn to_modfile(&self, file: File) -> std::io::Result<()> {
            let writer = &mut BufWriter::new(file);
            self.r_to_modfile(writer)
        }
    }
    // Create a file to represent root
    let mut root = Node::new("", None, true);
    // Add each outfile
    for path in outfiles {
        root.add_file(&path);
    }
    // Turn it into a modfile
    root.to_modfile(modfile)?;
    // Return ok
    Ok(())
}

fn build_protos() -> Result<(), ()> {
    let outdir = Path::new(RELATIVE_PROTO_OUT_DIR_PATH);
    info!("Selecting output: {:?}", outdir);

    if !outdir.exists() {
        println!("cargo:rerun-if-changed={}", RELATIVE_PROTO_DIR_PATH);
    }

    env_logger::builder().filter_level(LevelFilter::Info).init();

    info!("[aptos-etl] Starting aptos-etl build...");

    let mut config = prost_build::Config::new();

    config.message_attribute(".aptos", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.enum_attribute(".aptos", "#[derive(serde::Serialize, serde::Deserialize)]");

    info!("[aptos-etl] Searching for proto files...");
    let dir = Path::new(RELATIVE_PROTO_DIR_PATH);
    info!(
        "[aptos-etl] Searching starting in proto root dir: {:?}",
        dir
    );
    let proto_files = match collect_proto_files(dir) {
        Ok(proto_files) => {
            info!("[aptos-etl] Collected {} proto files", proto_files.len());
            proto_files
        }
        Err(error) => {
            error!(
                "[aptos-etl] Failed to build protos for aptos-etl due to: {}",
                error
            );
            panic!(
                "[aptos-etl] Failed to build protos for aptos-etl due to: {}",
                error
            );
        }
    };

    config.out_dir(outdir);
    info!("Locating or creating output: {:?}", outdir);
    match create_dir_all(outdir) {
        Ok(_) => info!("[aptos-etl] Successfully located/created output directory."),
        Err(error) => {
            error!(
                "[aptos-etl] Error when creating codegen output dir: {}",
                error
            );
            panic!(
                "[aptos-etl] Error when creating codegen output dir: {}",
                error
            );
        }
    }

    info!("Attempting to build from protobufs");
    let protos_results = config.compile_protos(&proto_files, &[RELATIVE_PROTO_DIR_PATH]);

    match protos_results {
        Ok(_) => info!("Protos for aptos-etl created"),
        Err(error) => {
            error!(
                "[aptos-etl] Failed to build protos for aptos-etl: {}",
                error
            );
            panic!(
                "[aptos-etl] Failed to build protos for aptos-etl: {}",
                error
            );
        }
    }

    info!("[aptos-etl] Collecting all outputted-rust code");
    let outfiles = match collect_output_rs_files(outdir, true) {
        Ok(outfiles) => outfiles,
        Err(error) => {
            error!("Failed to get output rs files: {:?}", error);
            panic!("Failed to get output rs files: {:?}", error);
        }
    };

    let outmodfilepath = outdir.join("mod.rs");
    let outmodfile = match File::create(outmodfilepath.clone()) {
        Ok(file) => file,
        Err(error) => {
            error!("Failed to create outmodfile: {:?}", error);
            panic!("Failed to create outmodfile: {:?}", error);
        }
    };
    match create_mod(outfiles, outmodfile) {
        Ok(_) => {}
        Err(error) => {
            error!("Failed to output outmodfile: {:?}", error);
            panic!("Failed to output outmodfile: {:?}", error);
        }
    };

    // Assume your mod.rs is located at "src/my_module/mod.rs"
    let output = Command::new("rustfmt")
        .arg(outmodfilepath.to_str().unwrap()) // Convert PathBuf to &str
        .output()
        .expect("Failed to execute rustfmt");

    if !output.status.success() {
        // You can handle errors more gracefully depending on your needs
        panic!(
            "rustfmt failed with error: {:?}",
            String::from_utf8(output.stderr)
        );
    }

    info!("Successfully finalized build-script");

    Ok(())
}
