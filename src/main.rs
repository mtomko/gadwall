use clap::Parser;
use std::collections::HashMap;
use std::process;
use tokio::fs::File;
use tokio_stream::StreamExt;
mod dna;
use core::fmt;

#[derive(Parser, Debug)]
#[command(author = "Mark Tomko <me@marktomko.org>")]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'c', long, value_name = "CSV")]
    conditions: String,
    #[arg(short = '1', long, value_name = "FASTQ")]
    dmux: String,
    #[arg(short = '2', long, value_name = "FASTQ")]
    data: String,
    #[arg(short, long, value_name = "DIR")]
    output_dir: String,
}

#[derive(Debug, Clone)]
struct CsvError {
    message: String,
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

async fn load_conditions(file_in: &str) -> Result<HashMap<String, String>, CsvError> {
    //let mut conditions = HashMap::new();
    let mut rdr = csv_async::AsyncReaderBuilder::new()
        .has_headers(false)
        .create_reader(File::open(file_in).await.map_err(|_| CsvError {
            message: "Unable to create reader".to_string(),
        })?);
    let rs: Result<Vec<(String, String)>, CsvError> = rdr
        .records()
        .map(|r| match r {
            Ok(record) => match (record.get(0), record.get(1)) {
                (Some(barcode), Some(name)) => {
                    if dna::is_strict_dna(barcode) {
                        Ok((barcode.to_string(), name.to_string()))
                    } else {
                        Err(CsvError {
                            message: "invalid barcode".to_string(),
                        })
                    }
                }
                _ => Err(CsvError {
                    message: "doh".to_string(),
                }),
            },
            Err(_) => Err(CsvError {
                message: "bad csv".to_string(),
            }),
        })
        .collect()
        .await;

    rs.map(|mappings| mappings.into_iter().collect())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Err(err) = load_conditions(&args.conditions).await {
        eprint!("Error loading conditions: {}", err);
        process::exit(1)
    }
}
