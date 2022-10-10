use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::process;
use tokio::fs::File;
use tokio_stream::StreamExt;
mod dna;

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

async fn load_conditions(file_in: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut conditions = HashMap::new();
    let mut rdr = csv_async::AsyncReaderBuilder::new()
        .has_headers(false)
        .create_reader(File::open(file_in).await?);
    let mut records = rdr.records();
    while let Some(record) = records.next().await {
        let record = record?;
        match (record.get(0), record.get(1)) {
            (Some(barcode), Some(name)) if dna::is_strict_dna(barcode) => {
                conditions.insert(barcode.to_string(), name.to_string())
            }
            _ => None,
        };
    }
    Ok(conditions)
}

async fn run(args: Args) {
    if let Err(err) = load_conditions(&args.conditions).await {
        eprint!("Error loading conditions: {}", err);
        process::exit(1)
    }
}

fn main() {
    let args = Args::parse();
    tokio::runtime::Runtime::new().unwrap().block_on(run(args));
}
