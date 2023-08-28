use arrow_array::StringArray;
// uGe arrow::record_batch::RecordBatch;
// use arrow_array::RecordBatch;
// use arrow_array::{ArrayRef, Int32Array};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
// use parquet::arrow::arrow_writer::ArrowWriter;
// use parquet::file::properties::WriterProperties;
// use parquet::arrow::arrow_reader;
use sqlparser::ast::{self, SelectItem, Statement};
use sqlparser::dialect::GenericDialect;
use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
// use std::sync::Arc;

// Top level parser
// Retuns Optional Table
// If table exists, returns table as requested in a select, or table with newly insterted data,
// or empty table

pub fn parse(sql: &str) -> Vec<Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>>> {
    // Generate AST, identify statement type, then pass off function to appropriate parser.
    println!("hello from parse");

    let unparsed_statements =
        sqlparser::parser::Parser::parse_sql(&GenericDialect {}, sql).unwrap();

    // let tables: Vec<Result<(String, Vec<String>), Box<dyn std::error::Error>>>;
    let mut tables = vec![];

    for statement in &unparsed_statements {
        // println!("statements: {:?}\n", statement);
        match statement {
            Statement::Query(query) => match *query.body.clone() {
                ast::SetExpr::Select(sel) => {
                    println!("found select: {:?}", sel);
                    tables.push(handle_select(&sel));
                }
                _ => println!("did not find select"),
            },
            _ => println!("query not found"),
        }
    }
    tables
}

fn handle_select(
    select_statement: &Box<sqlparser::ast::Select>,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    println!("\nHello from handle_select");
    let columns = &select_statement.projection;

    let mut txt_cols: Vec<&String> = vec![];
    let tables = &select_statement.from;
    let table = tables[0].relation.to_string();

    for column in columns {
        match column {
            SelectItem::UnnamedExpr(exp) => {
                match exp {
                    ast::Expr::Identifier(ident) => {
                        println!("found ident: {:?}", ident.value);
                        let val: &String = &ident.value;
                        txt_cols.push(val);
                    }
                    _ => println!("did not find ident"),
                }
                println!("found unamed exp: {:?}", exp);
            }
            SelectItem::Wildcard(wild) => {
                println!("found wildcard: {}", wild);
            }
            _ => println!("found neither exp nor wildcard"),
        }
    }

    println!("columns: {:?}\n", columns);
    println!("txt_cols: {:?}\n", txt_cols);
    println!("table: {:?}\n ", table);

    let res = get_table(&table, "teachers.parquet", &txt_cols);
    res
}

fn get_table(
    table_name: &str,
    path: &str,
    columns: &Vec<&String>,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    println!("\nhello from get_table");
    println!(
        "table_name: {}, path: {}, columns: {:?}",
        table_name, path, columns
    );

    let file = File::open(path)?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(file).unwrap();
    // let table_schema = builder.schema();

    let mut reader = builder.build().unwrap();

    let record_batch = reader.next().unwrap().unwrap();

    let mut return_table = HashMap::new();

    for col in columns {
        let recordbatch_column = record_batch.column_by_name(col);
        // return_table.insert(col.to_string(), Vec::<String>::new());
        let mut col_vec = Vec::<String>::new();
        for i in 0..record_batch.num_rows() {
            // println!("Reading col {:?} and row {:?}", col, i);
            // println!("recordbatch_column: {:?}", recordbatch_column);
            if let Some(arc_array) = recordbatch_column {
                if let Some(str_array) = arc_array.as_any().downcast_ref::<StringArray>() {
                    println!("Some(str_array): {:?}", str_array.value(i));
                    col_vec.push(str_array.value(i).to_string());
                    // if let col_hash = return_table.get(col.to_string()) {}
                    // return_table.insert(col.to_string(), str_array.value(i));
                    // return_table[col.to_string()].push(1);
                }
            } else {
                continue;
            }
        }
        return_table.insert(col.to_string(), col_vec);
    }

    println!("return_table: {:?}", return_table);
    Ok(return_table)
}
