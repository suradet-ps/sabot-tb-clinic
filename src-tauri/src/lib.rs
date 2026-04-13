mod commands;
mod db;
mod models;

use commands::settings::MySqlState;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      let app_handle = app.handle().clone();
      tauri::async_runtime::block_on(async move {
        let app_data_dir = app_handle
          .path()
          .app_data_dir()
          .expect("Failed to get app data dir");
        std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");

        let db_path = app_data_dir.join("tb_clinic.db");
        let db_url = format!(
          "sqlite://{}?mode=rwc",
          db_path.to_str().expect("db path is not valid UTF-8")
        );

        let sqlite_pool = SqlitePoolOptions::new()
          .max_connections(5)
          .connect(&db_url)
          .await
          .expect("Failed to connect to SQLite");

        sqlx::migrate!("./migrations")
          .run(&sqlite_pool)
          .await
          .expect("Failed to run SQLite migrations");

        // ── Auto-connect to MySQL using persisted credentials ───────────────
        // If no config is saved, or the saved config fails to connect, we start
        // with mysql_state = None and let the user connect manually via Settings.
        let mysql_pool_initial: Option<sqlx::MySqlPool> =
          match crate::commands::settings::load_config_from_sqlite(&sqlite_pool).await {
            Ok(Some(config)) => {
              let url = format!(
                "mysql://{}:{}@{}:{}/{}",
                config.username, config.password, config.host, config.port, config.database
              );
              match MySqlPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
              {
                Ok(pool) => {
                  println!(
                    "[sabot] Auto-connected to MySQL ({}:{})",
                    config.host, config.port
                  );
                  Some(pool)
                }
                Err(e) => {
                  eprintln!("[sabot] Auto-connect to MySQL failed: {e}");
                  None
                }
              }
            }
            Ok(None) => {
              // No saved config — normal first-run situation, not an error
              None
            }
            Err(e) => {
              eprintln!("[sabot] Failed to load saved DB config: {e}");
              None
            }
          };

        app_handle.manage(sqlite_pool);

        let mysql_state: MySqlState = Arc::new(Mutex::new(mysql_pool_initial));
        app_handle.manage(mysql_state);
      });
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::screening::search_tb_patients,
      commands::screening::get_dispensing_history,
      commands::patients::enroll_patient,
      commands::patients::get_active_patients,
      commands::patients::get_patient_detail,
      commands::patients::discharge_patient,
      commands::patients::get_discharged_patients,
      commands::followups::add_followup,
      commands::followups::update_treatment_phase,
      commands::alerts::get_patient_alerts,
      commands::settings::test_mysql_connection,
      commands::settings::connect_mysql,
      commands::settings::get_mysql_status,
      commands::settings::backup_sqlite,
      commands::settings::save_db_config,
      commands::settings::load_db_config,
      commands::settings::delete_db_config,
      commands::appointments::get_appointments,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
