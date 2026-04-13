use crate::commands::settings::MySqlState;
use crate::db;
use crate::models::patient::AppointmentRecord;
use tauri::State;

/// Fetch upcoming TB clinic appointments from HOSxP `oapp` (clinic = `009`).
///
/// `days_ahead` controls how far into the future to look (default: 30 days).
/// Returns an error string when the MySQL connection is not available.
#[tauri::command]
pub async fn get_appointments(
  mysql: State<'_, MySqlState>,
  days_ahead: Option<i64>,
) -> Result<Vec<AppointmentRecord>, String> {
  let guard = mysql.lock().await;
  match &*guard {
    None => Err("MySQL ยังไม่ได้เชื่อมต่อ".to_string()),
    Some(pool) => {
      let days = days_ahead.unwrap_or(30);
      db::mysql::get_tb_appointments(pool, days)
        .await
        .map_err(|e| e.to_string())
    }
  }
}
