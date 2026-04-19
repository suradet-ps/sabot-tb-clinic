mod commands;
mod db;
mod models;

use commands::settings::MySqlState;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_dialog::init())
    .setup(|app| {
      let app_handle = app.handle().clone();

      // ── Step 1: SQLite — synchronous, completes in < 10 ms ─────────────
      // We use block_on here because SQLite is a local file operation.
      // This guarantees that sqlite_pool is managed before the event loop
      // starts, so Vue can safely call SQLite-backed commands immediately
      // after mount without any race condition.
      let sqlite_pool = tauri::async_runtime::block_on(async {
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

        let pool = SqlitePoolOptions::new()
          .max_connections(5)
          .connect(&db_url)
          .await
          .expect("Failed to connect to SQLite");

        sqlx::migrate!("./migrations")
          .run(&pool)
          .await
          .expect("Failed to run SQLite migrations");

        pool
      });

      // Register SQLite immediately so Vue commands work as soon as the
      // first frame renders.
      app_handle.manage(sqlite_pool.clone());

      // ── Step 2: MySQL — pre-register as None, connect asynchronously ───
      // MySQL auto-connect can take several seconds when the server is
      // behind Tailscale or temporarily unreachable.  We pre-register the
      // state as None so the event loop (and Vue) can start without waiting,
      // then fill it in once the connection is established.
      let mysql_state: MySqlState = Arc::new(Mutex::new(None));
      app_handle.manage(Arc::clone(&mysql_state));

      // ── Step 3: Background task — MySQL connect + splash-done signal ────
      let app_handle_clone = app_handle.clone();

      tauri::async_runtime::spawn(async move {
        // ── 3a. Update splash status ──────────────────────────────────────
        let _ = app_handle_clone.emit("splash-status", "กำลังโหลดฐานข้อมูล...");

        // ── 3b. Attempt MySQL auto-connect ────────────────────────────────
        let connect_result = crate::commands::settings::load_config_from_sqlite(&sqlite_pool).await;

        match connect_result {
          Ok(Some(config)) => {
            let _ = app_handle_clone.emit("splash-status", "กำลังเชื่อมต่อ MySQL...");

            let url = format!(
              "mysql://{}:{}@{}:{}/{}",
              config.username, config.password, config.host, config.port, config.database
            );

            // Hard-cap the auto-connect attempt at 8 seconds so startup is
            // never indefinitely blocked when the server is unreachable.
            let pool_result = tokio::time::timeout(
              std::time::Duration::from_secs(8),
              MySqlPoolOptions::new().max_connections(5).connect(&url),
            )
            .await;

            match pool_result {
              Ok(Ok(pool)) => {
                println!(
                  "[sabot] Auto-connected to MySQL ({}:{})",
                  config.host, config.port
                );
                let mut guard = mysql_state.lock().await;
                *guard = Some(pool);
                let _ = app_handle_clone.emit("splash-status", "เชื่อมต่อสำเร็จ ✓");
              }
              Ok(Err(e)) => {
                eprintln!("[sabot] Auto-connect to MySQL failed: {e}");
                let _ = app_handle_clone.emit("splash-status", "เชื่อมต่อล้มเหลว (ใช้งานออฟไลน์ได้)");
              }
              Err(_) => {
                eprintln!("[sabot] MySQL auto-connect timed out after 8 s");
                let _ = app_handle_clone.emit("splash-status", "เชื่อมต่อหมดเวลา (ใช้งานออฟไลน์ได้)");
              }
            }
          }
          Ok(None) => {
            // No saved config — normal on first run, nothing to do.
            let _ = app_handle_clone.emit("splash-status", "พร้อมใช้งาน (ยังไม่ตั้งค่า MySQL)");
          }
          Err(e) => {
            eprintln!("[sabot] Failed to load saved DB config: {e}");
            let _ = app_handle_clone.emit("splash-status", "โหลดการตั้งค่าล้มเหลว");
          }
        }

        // ── 3c. Minimum splash display time (visual comfort) ─────────────
        // The splash overlay is now removed by Vue's onMounted lifecycle
        // (App.vue) after the frontend finishes its own init sequence.
        // This avoids the race condition where splash-done fires before the
        // Vue listener is registered (especially in dev mode with Vite).
        // We keep a short sleep so status text remains readable briefly.
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
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
      commands::mapping::get_mapping_patients,
      commands::mapping::get_mapping_summary,
      commands::mapping::geocode_patient_address,
      commands::mapping::batch_geocode_patients,
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
