use anyhow::{anyhow, Context, Result};
use bb8_diesel::{DieselConnection, DieselConnectionManager};
use rand::{seq::SliceRandom, thread_rng};
use std::sync::Arc;
use tbot::{
    contexts::methods::ChatMethods,
    types::{
        chat::{member::Status, Permissions},
        dice::{Dice, Kind::Darts},
    },
    Bot,
};
use tracing::{error, info};

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use bb8::{Builder, Pool};
use dotenv::dotenv;
use std::env;

pub mod models;
pub mod schema;

// use self::diesel::prelude::*;
// use self::diesel_demo::*;
use self::models::*;


pub async fn establish_connection() -> Pool<DieselConnectionManager<SqliteConnection>> {    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = bb8_diesel::DieselConnectionManager::<SqliteConnection>::new(database_url);
    let pool = bb8::Pool::builder().build(manager).await.unwrap();
    pool
}

pub fn create_entry(
    conn: &SqliteConnection,
    chat_id: i64,
    message_id: i64,
    cron_specifier: &str,
    message: &str,
) -> usize {
    use schema::cron_entries;
    let new_entry = NewCronEntry {
        chat_id,
        message_id,
        cron_specifier,
        message,
    };

    diesel::insert_into(cron_entries::table)
        .values(&new_entry)
        .execute(conn)
        .expect("Error saving new cron entry")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let connection_pool = establish_connection().await;

    let mut bot_loop = Bot::new(env::var("BOT_TOKEN").expect("BOT_TOKEN must be set")).event_loop();

    bot_loop.dice(|context| async move {
        handle_dice(&context).await.unwrap_or_else(|e| {
            error!("Dice handled unsuccessfully: {}", e);
        });
    });

    bot_loop.command(
        "add",
        |context: Arc<tbot::contexts::Command<tbot::contexts::Text>>| async move {
            handle_add(&context, connection_pool).await.unwrap_or_else(|e| {
                error!("add handled unsuccessfully: {}", e);
            });
        },
    );
    bot_loop
        .polling()
        .start()
        .await
        .map_err(|e| anyhow!("Error setting up polling loop: {:?}", e))?;

    Ok(())
}

async fn handle_add(
    ctx: &tbot::contexts::Command<tbot::contexts::Text>,
    connection_pool: Pool<DieselConnectionManager<SqliteConnection>>,
) -> Result<()> {
    println!("Nachricht {} form {}", ctx.text.value, ctx.message_id);
    create_entry(&connection_pool.get().await.expect("Failed to sqlite connection"), ctx.chat.id.0, ctx.message_id.0 as i64, ctx.text.value.as_str(), ctx.text.value.as_str());
    ctx.send_message_in_reply(
        "Guter Wurf! Aber jetzt genug geübt, probier dein Glück in der Hauptgruppe!",
    )
    .call()
    .await?;
    Ok(())
}

async fn handle_dice(ctx: &tbot::contexts::Dice) -> Result<()> {
    if let Dice {
        kind: Darts,
        value: _b,
        ..
    } = ctx.dice
    {
        let user = ctx.from.as_ref().context("Dice sent by nobody")?;

        info!(
            "{} won!",
            user.username.as_deref().unwrap_or("anonymous user")
        );

        if ctx.chat.kind.is_supergroup() {
            let member = ctx.get_chat_member(user.id).call().await?;

            tokio::time::delay_for(std::time::Duration::from_secs(30)).await;

            match member.status {
                Status::Administrator { .. } | Status::Creator { .. } => {
                    ctx.send_message_in_reply("Ich kann Dich zwar nicht muten, aber sei doch bitte so lieb und halt trotzdem eine Woche lang die Fresse.").call().await?;
                }
                Status::Member | Status::Restricted { .. } => {
                    let permissions = Permissions::new().can_send_messages(false);
                    match ctx
                        .restrict_chat_member(user.id, permissions)
                        .until_date(ctx.date.saturating_add(7 * 24 * 60 * 60))
                        .call()
                        .await
                    {
                        Err(_e) => {
                            let msg = *["Gebt mir Admin-Rechte!",
                                    "Ich hab zwar keine Rechte dich zu Muten, aber sei bitte trotzdem still.",
                                    "Wann kann ich endlich Leute muten?",
                                ]
                                    .choose(&mut thread_rng())
                                    .unwrap();
                            ctx.send_message_in_reply(msg).call().await?;
                        }
                        Ok(_t) => {
                            let msg = *[
                                "Jawollo!",
                                "Gewinner, Gewinner, Huhn Abendessen!",
                                "Viel Spaß bei einer Woche Urlaub von RWTH Informatik!",
                                "endlich",
                            ]
                            .choose(&mut thread_rng())
                            .unwrap();
                            ctx.send_message_in_reply(msg).call().await?;
                        }
                    };
                }
                Status::Left { .. } => {
                    ctx.send_message_in_reply("Schade dass Du schon weg bist. Ich werde Deinen Gewinn aufbewahren und einlösen, wenn Du uns wieder besuchst!").call().await?;
                }
                _ => {}
            }
        } else {
            tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
            ctx.send_message_in_reply(
                "Guter Wurf! Aber jetzt genug geübt, probier dein Glück in der Hauptgruppe!",
            )
            .call()
            .await?;
        }
    };

    Ok(())
}
