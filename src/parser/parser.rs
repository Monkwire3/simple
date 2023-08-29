use arrow_array::{Int32Array, StringArray};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use sqlparser::ast::{self, SelectItem, Statement};
use sqlparser::dialect::GenericDialect;
use std::collections::HashMap;
use std::fs::File;

#[derive(Debug)]
pub enum TableValue {
    StringValue(String),
    Int32Value(i32),
}
// Parse function returns a vec of the results of all SQL Statements. All successful statement
// results return tables.
pub fn parse(
    sql: &str,
) -> Vec<Result<HashMap<String, Vec<TableValue>>, Box<dyn std::error::Error>>> {
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
) -> Result<HashMap<String, Vec<TableValue>>, Box<dyn std::error::Error>> {
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
            SelectItem::Wildcard(_w) => {
                return get_table(&table, "teachers.parquet", &vec![], true)
            }
            _ => println!("found neither exp nor wildcard"),
        }
    }

    let res = get_table(&table, "teachers.parquet", &txt_cols, false);
    res
}

fn get_table(
    _table_name: &str,
    path: &str,
    columns: &Vec<&String>,
    wildcard: bool,
) -> Result<HashMap<String, Vec<TableValue>>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(file).unwrap();

    let mut reader = builder.build().unwrap();

    let record_batch = reader.next().unwrap().unwrap();

    let schema_ref = record_batch.schema();

    let mut return_table = HashMap::new();

    let all_columns: Vec<&String> = schema_ref.fields().iter().map(|x| x.name()).collect();
    let columns = if wildcard { &all_columns } else { columns };

    for col in columns {
        let col_index = schema_ref.index_of(col);
        let recordbatch_column = record_batch.column_by_name(col);
        let col_type = schema_ref.field(col_index.unwrap()).data_type();
        let mut col_vec = Vec::<TableValue>::new();
        for i in 0..record_batch.num_rows() {
            match col_type {
                arrow::datatypes::DataType::Int32 {} => {
                    if let Some(arc_array) = recordbatch_column {
                        if let Some(int_array) = arc_array.as_any().downcast_ref::<Int32Array>() {
                            col_vec.push(TableValue::Int32Value(int_array.value(i as usize)))
                        }
                    }
                }
                arrow::datatypes::DataType::Utf8 {} => {
                    if let Some(arc_array) = recordbatch_column {
                        if let Some(str_array) = arc_array.as_any().downcast_ref::<StringArray>() {
                            col_vec.push(TableValue::StringValue(
                                str_array.value(i as usize).to_string(),
                            ));
                        }
                    }
                }
                _ => println!("col_type is neither"),
            }
        }
        return_table.insert(col.to_string(), col_vec);
    }
    Ok(return_table)
}
