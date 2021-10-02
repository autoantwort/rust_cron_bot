
use diesel::sql_types::{BigInt};

use super::schema::cron_entries;

#[derive(Queryable)]
pub struct CronEntry {
    pub chat_id: BigInt,
    pub message_id: BigInt,
    pub cron_specifier: String,
    pub message: String,
}

#[derive(Insertable)]
#[table_name = "cron_entries"]
pub struct NewCronEntry<'a> {
    pub chat_id: i64,
    pub message_id: i64,
    pub cron_specifier: &'a str,
    pub message: &'a str,
}