use arrow::datatypes::SchemaRef;
use arrow_array::Int32Array;
use arrow_array::StringArray;
use arrow_schema::Schema;
use parquet::data_type::Int32Type;
use arrow_array::ArrayRef;
use parquet::file::serialized_reader::SerializedFileReader;
use parquet::file::writer::SerializedFileWriter;
use parquet::file::writer::SerializedRowGroupWriter;
use parquet::file::writer::TrackedWrite;
use parquet::schema::types::SchemaDescriptor;
use parquet::{file::properties::WriterProperties, schema::types::Type};
use std::path::Path;
use std::sync::Arc;
use std::{
fs::File,
io::{Seek, SeekFrom},
};

pub fn insert(path: &str) -> Result<(), std::io::Error> {
    // Open file as read and write
    let mut file_read = File::options()
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)?;


    let mut file_write = file_read.try_clone().unwrap();

    // Create a `SchemaDescriptor` from the parsed `Type`.
    let reader = SerializedFileReader::new(file_read).unwrap();
    let parquet_metadata = parquet::file::reader::FileReader::metadata(&reader);
    let schema = parquet_metadata.file_metadata().schema();
    println!("schema: {:?}", schema);

    
    //
    // Set next write pointer 8 bytes from the end. This overwrites the metadata at the end.
    file_write.seek(SeekFrom::End(-8))?;

    // Create group_ writer, which can write to 
    let mut writer = SerializedFileWriter::new(
        file_write,
        Arc::new(schema.clone()),
        Arc::new(WriterProperties::new()),
    );

    // Manually add appropriate rows
    let new_rows = vec![
        Arc::new(Int32Array::from(vec![1, 2, 3])) as ArrayRef,
        Arc::new(StringArray::from(vec!["Jones", "Johson", "Smith"])) as ArrayRef,
        Arc::new(StringArray::from(vec!["spanish", "science", "english"])) as ArrayRef,
    ];

    //  Boilerplate column-writer
    // let mut column = writer.next_column();
    // if let Some(mut col_writer) = column.unwrap() {
    //     let values: [i32; 5] = [1, 2, 3, 4, 5];
    //
    //     col_writer
    //         .typed::<Int32Type>()
    //         .write_batch(&values, None, None)
    //         .unwrap();
    //         col_writer.close().unwrap()
    // }
    // writer.close().unwrap();
    //
    Ok(())
}



fn write_column() {}

fn make_row_group() {}

fn write_to_file() {}