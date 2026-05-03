use std::env;

use crate::error::Result;

#[derive(Debug)]
pub struct Environment {
    pub table_name: String,
}

impl Environment {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            table_name: env::var("TABLE_NAME").expect("table_name"),
        })
    }
}
