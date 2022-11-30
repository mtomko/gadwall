mod dna;
use clap::Parser;
use core::fmt;
use fastq::{parse_path, Record};
use itermap::IterMap;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::process;
use std::str;
use tokio_stream::StreamExt;

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
        .create_reader(tokio::fs::File::open(file_in).await.map_err(|_| CsvError {
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
                    message: "invalid conditions file".to_string(),
                }),
            },
            Err(_) => Err(CsvError {
                message: "invalid csv".to_string(),
            }),
        })
        .collect()
        .await;

    rs.map(|mappings| mappings.into_iter().collect())
}

fn read_fastqs(file_in1: &str, file_in2: &str, mut conditions: HashMap<String, File>) {
    parse_path(Some(file_in1), |dmux| {
        parse_path(Some(file_in2), |data| {
            let mut dmux_iter = dmux.ref_iter();
            let mut data_iter = data.ref_iter();
            while let (Some(dmux_rec), Some(data_rec)) = (dmux_iter.get(), data_iter.get()) {
                let bc_bytes = &dmux_rec.seq().to_ascii_uppercase();
                let barcode = str::from_utf8(bc_bytes).expect("ASCII");
                let writer = conditions.get_mut(barcode);
                writer.map(|w| data_rec.write(w));
                dmux_iter.advance().expect("empty dmux");
                data_iter.advance().expect("empty data");
            }
        })
        .expect("bad data")
    })
    .expect("bad dmux");
}

fn condition_writers(
    output_dir: &str,
    conditions: HashMap<String, String>,
) -> HashMap<String, File> {
    let p = Path::new(output_dir);
    conditions
        .into_iter()
        .map_values(|v| {
            //let mut w =
            std::fs::File::create(p.join(v)).expect("Should have been able to create file")
            //&mut w
        })
        .collect()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match load_conditions(&args.conditions).await {
        Ok(conditions) => read_fastqs(
            &args.dmux,
            &args.data,
            condition_writers(&args.output_dir, conditions),
        ),
        Err(err) => {
            eprint!("Error loading conditions: {}", err);
            process::exit(1)
        }
    }
}
