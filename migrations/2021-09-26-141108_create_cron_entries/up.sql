CREATE TABLE cron_entries (
  chat_id INTEGER NOT NULL,
  message_id INTEGER NOT NULL,
  cron_specifier TEXT NOT NULL,
  message TEXT NOT NULL,
  PRIMARY KEY (chat_id, message_id)
)