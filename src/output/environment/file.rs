use dotenvy;
use once_cell::sync::OnceCell;

/// The environment key leading to the path for the output directory
pub const OUTPUT_DIR_ENVKEY: &str = "OUTPUT_DIR";
/// Stores the Output Directory
pub static OUTPUT_DIR: OnceCell<String> = OnceCell::new();
/// Returns the output directory from the .env
pub fn get_output_dir() -> &'static String {
    OUTPUT_DIR.get_or_init(|| {
        dotenvy::var(OUTPUT_DIR_ENVKEY)
            .unwrap_or_else(|_| panic!("{} should exist in .env file", OUTPUT_DIR_ENVKEY))
            .parse::<String>()
            .unwrap()
    })
}
