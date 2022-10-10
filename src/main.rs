use std::collections::HashMap;
use std::error::Error;
use std::process;
use tokio::fs::File;
use tokio_stream::StreamExt;

async fn load_conditions(file_in: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut conditions = HashMap::new();
    let mut rdr = csv_async::AsyncReader::from_reader(File::open(file_in).await?);
    let mut records = rdr.records();
    while let Some(record) = records.next().await {
        let record = record?;
        match (record.get(0), record.get(1)) {
            (Some(barcode), Some(name)) => conditions.insert(barcode.to_string(), name.to_string()),
            _ => None,
        };
    }
    Ok(conditions)
}

#[tokio::main]
async fn main() {
    if let Err(err) = load_conditions("conditions.csv").await {
        eprint!("Error loading conditions: {}", err);
        process::exit(1)
    }
}
