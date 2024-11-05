use std::path::Path;

use super::*;
use tests::processing::tables::all::extract_records;

/// Example Transformation Directory is the directory where the expected
pub const EXAMPLE_TRANSFORMATION_DIR: &str = "./tests/examples/";

#[derive(Debug, Clone)]
pub enum ExampleRangeError {
    NotWithinRange(u64),
    TxNotFile(PathBuf),
}

impl std::fmt::Display for ExampleRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Wraps around a directory representing an Example Range.
pub struct ExampleRange {
    /// The directory that the [ExampleRange] exists in
    dir: PathBuf,
    /// The range
    range: (u64, u64),
    /// The name
    name: String,
}

impl ExampleRange {
    /// Returns the name of the [ExampleRange]
    #[inline]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    /// Returns the start of the [ExampleRange]
    #[inline]
    pub fn start(&self) -> u64 {
        self.range.0
    }
    /// Returns the end of the [ExampleRange]
    #[inline]
    pub fn end(&self) -> u64 {
        self.range.1
    }

    #[inline]
    pub fn get_tx(
        &self,
        version: u64,
    ) -> Result<aptos_protos::transaction::v1::Transaction, ExampleRangeError> {
        let path = self.dir.join(format!("txs/{}.pb", version));

        // Validate path exists, and is a file.
        if !path.exists() {
            return Err(ExampleRangeError::NotWithinRange(version));
        } else if !path.is_file() {
            return Err(ExampleRangeError::TxNotFile(path));
        }

        match load_tx(&path) {
            Ok(tx) => Ok(tx),
            Err(err) => {
                panic!("Failed to `load_tx` due to extraction error: {}", err);
            }
        }
    }

    #[inline]
    pub fn get_records(
        &self,
        version: u64,
    ) -> Result<proto_codegen::aptos::records::Records, ExampleRangeError> {
        let path = self.dir.join(format!("records/{}.pb", version));

        // Validate path exists, and is a file.
        if !path.exists() {
            return Err(ExampleRangeError::NotWithinRange(version));
        } else if !path.is_file() {
            return Err(ExampleRangeError::TxNotFile(path));
        }

        // Load the records, if something fails here we panic.
        match load_records(&path) {
            Ok(records) => Ok(records),
            Err(err) => {
                panic!("Failed to `load_tx` due to extraction error: {}", err);
            }
        }
    }

    /// Creates [Vec]<[ExampleRange]>.  If does not provide directory path, utilized
    /// [EXAMPLE_TRANSFORMATION_DIR]
    pub fn from_dirpath_mult(dirpath: Option<&PathBuf>) -> Vec<Self> {
        // Create a directory path.
        let dirpath = match dirpath {
            Some(dirpath) => dirpath.clone(),
            None => PathBuf::from(EXAMPLE_TRANSFORMATION_DIR),
        };
        // Creates an examples
        let mut examples = Vec::new();
        // Go through the directory and make each one
        match std::fs::read_dir(dirpath) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(dir_entry) => {
                            let path = dir_entry.path();
                            if !path.is_dir() {
                                info!("Skipping {:?} as it is a dir", path);
                                continue;
                            }
                            examples.push(Self::from_dirpath(&path));
                        }
                        Err(err) => {
                            panic!("An error occured: {}", err);
                        }
                    }
                }
            }
            Err(err) => {
                panic!("An error occured: {}", err)
            }
        }

        examples
    }

    /// Creates an [ExampleRange] from the a directory path.
    pub fn from_dirpath(dirpath: &Path) -> Self {
        // Creates the directory name
        let dirname = dirpath
            .file_name()
            .expect("Directory paths must be utf-8")
            .to_str()
            .expect("Directory paths must be utf-8");
        // Extract the key values
        let (name, start, end) = {
            let mut parts = dirname.rsplitn(3, '_');
            let end = parts
                .next()
                .expect("Directory should be {name}_{start}_{end}");
            let start = parts
                .next()
                .expect("Directory should be {name}_{start}_{end}");
            let name = parts.collect::<Vec<_>>().join("_");
            // Parse the start/end
            let start_v = start
                .parse::<u64>()
                .expect("Second to last part should be u64");
            let end_v = end
                .parse::<u64>()
                .expect("Second to last part should be u64");
            // Returns the name, start version, and end version
            (name, start_v, end_v)
        };
        // Returns the value
        Self {
            dir: dirpath.to_path_buf(),
            range: (start, end),
            name,
        }
    }
}

/// Attempts to extract and compares to what saved protobufs we have saved.  
///
/// An error can be raised if we fail to retrieve any transactions from the stream.
///
/// It can also explicitly state that the transactions do not match.
#[tokio::test]
pub async fn test_extractions() {
    println!(
        "Testing all extractions and validate that what is pulled from the server matches what
    we have stored."
    );
    // Iterate through all the example transactions
    for ex_range in ExampleRange::from_dirpath_mult(None) {
        // Initiate the start of the test
        println!("Test: {}", ex_range.name());
        // Extract the transactions
        println!("Extracting txs");
        let txs = match extract_txs(ex_range.start(), ex_range.end(), None).await {
            Ok(txs) => txs,
            Err(err) => panic!("Failed to extract transactions: {:?}", err),
        };
        // Iterate through the transactions
        println!("Iterating through txs");
        for tx in txs {
            // Get the version
            let version = tx.version;
            // Attempt to load it, then compare it to the saved transaction
            match ex_range.get_tx(version) {
                Ok(saved_tx) => assert_eq!(
                    saved_tx, tx,
                    "Transaction {} does not match the saved transaction",
                    version
                ),
                Err(err) => {
                    panic!("Failed to extract from ExampleRange: {}", err);
                }
            }
        }
    }
}

/// Evaluates saved test inputs and validates output.  This operates independent of the
/// any extraction, unlike [test_extractions] and [test_extract_range].  If this is successful
/// and [test_extractions] and [test_extract_range] are not, it is likely the transformation
/// process no longer operates given the current extraction data.
#[test]
pub fn test_transformations() {
    println!(
        "Testing transformation capabilities by using saved transactions and expected output 
    records"
    );

    // Iterate through all the example transactions
    for ex_range in ExampleRange::from_dirpath_mult(None) {
        println!(
            "Attempting to transform examples {} [{},{}]",
            ex_range.name(),
            ex_range.start(),
            ex_range.end()
        );
        // Iterate through the transactions
        for version in ex_range.start()..ex_range.end() {
            let tx = match ex_range.get_tx(version) {
                Ok(tx) => tx,
                Err(err) => {
                    panic!("Failed to retrieve saved tx: {}", err);
                }
            };

            let expected_output = match ex_range.get_records(version) {
                Ok(records) => records,
                Err(err) => panic!("Failed to retrieve saved records: {}", err),
            };

            let output = match extract_records(&tx) {
                Ok(output) => output,
                Err(err) => {
                    panic!("Error occured while transforming records: {}", err)
                }
            };

            assert_eq!(
                expected_output, output,
                "Output resources did not match: \n {:?} \n versus\n {:?}",
                expected_output, output
            );
        }
    }
}
