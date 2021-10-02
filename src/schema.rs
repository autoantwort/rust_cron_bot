table! {
    cron_entries (chat_id, message_id) {
        chat_id -> BigInt,
        message_id -> BigInt,
        cron_specifier -> Text,
        message -> Text,
    }
}
