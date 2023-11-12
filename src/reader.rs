use std::path::Path;

use polars::{io::avro::AvroReader, prelude::*};

pub fn read(path: &Path) -> PolarsResult<DataFrame> {
    let ext = path
        .extension()
        .expect("file extension should be specified");

    match ext.to_str().unwrap().to_lowercase().as_str() {
        "csv" => return read_csv(path, b','),
        "tsv" => return read_csv(path, b'\t'),
        "avro" => return read_avro(path),
        "parquet" => return read_parquet(path),
        _ => return Err(PolarsError::NoData("unsupported file extension".into())),
    }
}

fn read_csv(path: &Path, separator: u8) -> PolarsResult<DataFrame> {
    CsvReader::from_path(path)?
        .with_separator(separator)
        .infer_schema(Some(3))
        .has_header(true)
        .with_try_parse_dates(true)
        .finish()
}

fn read_parquet(path: &Path) -> PolarsResult<DataFrame> {
    let file = std::fs::File::open(path)?;
    ParquetReader::new(file)
        .use_statistics(true)
        .set_rechunk(true)
        .finish()
}

fn read_avro(path: &Path) -> PolarsResult<DataFrame> {
    let file = std::fs::File::open(path)?;
    AvroReader::new(file).finish()
}
