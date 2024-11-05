use dotenvy;
use log::warn;
use once_cell::sync::OnceCell;

/// The .env key to access the GCP credential json path
pub const GCP_CRED_JSON_PATH_ENVKEY: &str = "GOOGLE_APPLICATION_CREDENTIALS";
/// The GCP credentials Json Path
pub static GCP_CRED_JSON_PATH: OnceCell<Option<String>> = OnceCell::new();
/// Returns the GCP Credential path
pub fn get_gcp_credentials_json_path() -> &'static Option<String> {
    GCP_CRED_JSON_PATH.get_or_init(|| match dotenvy::var(GCP_CRED_JSON_PATH_ENVKEY) {
        Ok(ok) => {
            let key_path = ok.parse::<String>().unwrap();
            Some(key_path)
        }
        Err(e) => {
            warn!(
                "No env var {} in .env file ({}). Attempting to authenticate without it...",
                e, GCP_CRED_JSON_PATH_ENVKEY
            );
            None
        }
    })
}
