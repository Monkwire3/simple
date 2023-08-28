use arrow_array::StringArray;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use sqlparser::ast::{self, SelectItem, Statement};
use sqlparser::dialect::GenericDialect;
use std::collections::HashMap;
use std::fs::File;

// Parse function returns a vec of the results of all SQL Statements. All successful statement
// results return tables.
pub fn parse(sql: &str) -> Vec<Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>>> {
    // Separate SQL statements on ';'
    let statements = sqlparser::parser::Parser::parse_sql(&GenericDialect {}, sql).unwrap();

    // Create results table
    let mut tables = vec![];

    // Send each statement to the appropriate handler, then store the results (Result<Table, Err>
    // in tables)
    for statement in &statements {
        match statement {
            Statement::Query(query) => {
                if let ast::SetExpr::Select(sel) = &*query.body {
                    tables.push(handle_select(&sel));
                }
            }
            _ => println!("Only Statement::Query implemented"),
        }
    }
    tables
}

fn handle_select(
    select_statement: &Box<sqlparser::ast::Select>,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let columns = &select_statement.projection;

    let mut txt_cols: Vec<&String> = vec![];
    let tables = &select_statement.from;
    let table = tables[0].relation.to_string();

    for column in columns {
        match column {
            SelectItem::UnnamedExpr(exp) => {
                if let ast::Expr::Identifier(ident) = exp {
                    txt_cols.push(&ident.value);
                }
            }
            SelectItem::Wildcard(wild) => {
                println!("found wildcard: {}", wild);
            }
            _ => println!("found neither exp nor wildcard"),
        }
    }

    let res = get_table(&table, "teachers.parquet", &txt_cols);
    res
}

fn get_table(
    table_name: &str,
    path: &str,
    columns: &Vec<&String>,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(file).unwrap();
    // let table_schema = builder.schema();

    let mut reader = builder.build().unwrap();

    let record_batch = reader.next().unwrap().unwrap();

    let mut return_table = HashMap::new();

    for col in columns {
        let recordbatch_column = record_batch.column_by_name(col);
        let mut col_vec = Vec::<String>::new();
        for i in 0..record_batch.num_rows() {
            if let Some(arc_array) = recordbatch_column {
                if let Some(str_array) = arc_array.as_any().downcast_ref::<StringArray>() {
                    col_vec.push(str_array.value(i).to_string());
                }
            } else {
                continue;
            }
        }
        return_table.insert(col.to_string(), col_vec);
    }
    Ok(return_table)
}
