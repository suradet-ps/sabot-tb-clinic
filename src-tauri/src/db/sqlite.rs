use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};
use sqlx::SqlitePool;
use std::collections::HashMap;

use crate::models::patient::{EnrollmentInput, TbPatient};
use crate::models::treatment::{
  Followup, FollowupInput, Outcome, OutcomeInput, TreatmentPlan, TreatmentPlanUpdate,
};

// ─────────────────────────────────────────────────────────────────────────────
// Private date-arithmetic helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Add `months` calendar months to `date`, clamping the day to the last valid
/// day of the target month (e.g. 31 Jan + 1 month → 28 Feb).
fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
  let total = date.year() * 12 + date.month() as i32 - 1 + months;
  let year = total / 12;
  let month = (total % 12 + 1) as u32;
  let day = date.day().min(days_in_month(year, month));
  NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

fn days_in_month(year: i32, month: u32) -> u32 {
  match month {
    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    4 | 6 | 9 | 11 => 30,
    2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
    2 => 28,
    _ => 30,
  }
}

/// Parse a regimen string (e.g. `"2HRZE/4HR"`) into
/// `(intensive_months, continuation_months)`.
/// Falls back to `(2, 4)` for any unrecognised format.
fn parse_regimen_durations(regimen: &str) -> (i32, i32) {
  let mut parts = regimen.splitn(2, '/');
  let intensive = parts
    .next()
    .and_then(|s| s.chars().next())
    .and_then(|c| c.to_digit(10))
    .unwrap_or(2) as i32;
  let continuation = parts
    .next()
    .and_then(|s| s.chars().next())
    .and_then(|c| c.to_digit(10))
    .unwrap_or(4) as i32;
  (intensive, continuation)
}

// ─────────────────────────────────────────────────────────────────────────────
// tb_patients — read
// ─────────────────────────────────────────────────────────────────────────────

/// Return a map of `hn` → `status` for all patients in `tb_patients`.
/// Used to flag patients in the screening view (enrolled-active, discharged, etc.).
pub async fn get_enrolled_patients_map(pool: &SqlitePool) -> Result<HashMap<String, String>> {
  let rows: Vec<(String, String)> = sqlx::query_as("SELECT hn, status FROM tb_patients")
    .fetch_all(pool)
    .await?;
  Ok(rows.into_iter().collect())
}

/// Return every `hn` in `tb_patients`.
/// Thin wrapper around `get_enrolled_patients_map` kept for backward compatibility.
#[allow(dead_code)]
pub async fn get_all_enrolled_hns(pool: &SqlitePool) -> Result<Vec<String>> {
  sqlx::query_scalar("SELECT hn FROM tb_patients")
    .fetch_all(pool)
    .await
    .map_err(anyhow::Error::from)
}

/// Return all patients whose `status = 'active'`, newest enrolment first.
pub async fn get_active_patients(pool: &SqlitePool) -> Result<Vec<TbPatient>> {
  sqlx::query_as::<_, TbPatient>(
    "SELECT id, hn, enrolled_at, enrolled_by, status, tb_type,
                diagnosis_date, notes, created_at, updated_at
         FROM   tb_patients
         WHERE  status = 'active'
         ORDER  BY enrolled_at DESC",
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

/// Return all patients whose status is not `'active'` (completed, transferred,
/// died, defaulted). Used by the discharged patients view.
pub async fn get_discharged_patients(pool: &SqlitePool) -> Result<Vec<TbPatient>> {
  sqlx::query_as::<_, TbPatient>(
    "SELECT id, hn, enrolled_at, enrolled_by, status, tb_type,
                diagnosis_date, notes, created_at, updated_at
         FROM   tb_patients
         WHERE  status != 'active'
         ORDER  BY updated_at DESC",
  )
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

/// Return the `TbPatient` row for `hn`, or `None` when no record exists.
pub async fn get_patient_by_hn(pool: &SqlitePool, hn: &str) -> Result<Option<TbPatient>> {
  sqlx::query_as::<_, TbPatient>(
    "SELECT id, hn, enrolled_at, enrolled_by, status, tb_type,
                diagnosis_date, notes, created_at, updated_at
         FROM   tb_patients
         WHERE  hn = ?",
  )
  .bind(hn)
  .fetch_optional(pool)
  .await
  .map_err(anyhow::Error::from)
}

// ─────────────────────────────────────────────────────────────────────────────
// tb_patients — write
// ─────────────────────────────────────────────────────────────────────────────

/// Enrol a patient and create their initial treatment plan rows inside a
/// single transaction.
///
/// The **intensive** phase plan is set `is_current = 1`; the **continuation**
/// phase plan is pre-created with `is_current = 0`.  Pre-creating both rows
/// at enrolment ensures that `total_months` (derived by summing
/// `duration_months` across all plans) is correct from day one, without
/// waiting for the actual phase transition to occur.
pub async fn enroll_patient(pool: &SqlitePool, input: &EnrollmentInput) -> Result<i64> {
  let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
  let mut tx = pool.begin().await?;

  // ── Check whether a row already exists for this HN ───────────────────────
  let existing: Option<(i64, String)> =
    sqlx::query_as("SELECT id, status FROM tb_patients WHERE hn = ?")
      .bind(&input.hn)
      .fetch_optional(&mut *tx)
      .await?;

  let patient_id = if let Some((existing_id, existing_status)) = existing {
    if existing_status == "active" {
      // Patient is currently under treatment — block re-enrolment
      return Err(anyhow::anyhow!("ผู้ป่วยรายนี้ยังอยู่ในการรักษา"));
    }

    // ── RE-ENROLLMENT: patient was previously discharged / completed ──────
    // 1. Reactivate the patient row with the new enrolment details
    sqlx::query(
      "UPDATE tb_patients \
           SET enrolled_at    = ?1, \
               enrolled_by    = ?2, \
               tb_type        = ?3, \
               diagnosis_date = ?4, \
               notes          = ?5, \
               status         = 'active', \
               updated_at     = ?6 \
           WHERE hn = ?7",
    )
    .bind(&input.treatment_start_date)
    .bind(&input.enrolled_by)
    .bind(&input.tb_type)
    .bind(&input.diagnosis_date)
    .bind(&input.notes)
    .bind(&now)
    .bind(&input.hn)
    .execute(&mut *tx)
    .await?;

    // 2. Deactivate all previous treatment plan rows (kept for history)
    sqlx::query("UPDATE tb_treatment_plans SET is_current = 0 WHERE hn = ?")
      .bind(&input.hn)
      .execute(&mut *tx)
      .await?;

    existing_id
  } else {
    // ── NEW ENROLLMENT: no existing row — plain INSERT ────────────────────
    sqlx::query(
      "INSERT INTO tb_patients \
               (hn, enrolled_at, enrolled_by, status, tb_type, \
                diagnosis_date, notes, created_at, updated_at) \
           VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(&input.hn)
    .bind(&input.treatment_start_date)
    .bind(&input.enrolled_by)
    .bind(&input.tb_type)
    .bind(&input.diagnosis_date)
    .bind(&input.notes)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid()
  };

  // Create fresh intensive + continuation plan rows (shared by both paths)
  let start_date = NaiveDate::parse_from_str(&input.treatment_start_date, "%Y-%m-%d")
    .unwrap_or_else(|_| Local::now().date_naive());

  create_initial_plans(&mut tx, &input.hn, &input.regimen, start_date, &now).await?;

  tx.commit().await?;
  Ok(patient_id)
}

/// Insert intensive (current) and continuation (pending) plan rows for a
/// newly enrolled patient.
///
/// Drug icodes use Sarabosot Hospital codes from AGENTS.md:
/// - H+R+Z+E  →  `["1430104","1000265","1600004","1000258"]`
/// - H+R       →  `["1430104","1000265"]`
async fn create_initial_plans(
  tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
  hn: &str,
  regimen: &str,
  start_date: NaiveDate,
  now: &str,
) -> Result<()> {
  const HRZE: &str = r#"["H","R","Z","E"]"#;
  const HR: &str = r#"["H","R"]"#;

  let (intensive_months, continuation_months) = parse_regimen_durations(regimen);

  // ── Intensive phase (is_current = 1) ─────────────────────────────────────
  let intensive_end = add_months(start_date, intensive_months);

  sqlx::query(
    "INSERT INTO tb_treatment_plans
             (hn, regimen, phase, phase_start, phase_end_expected,
              drugs, duration_months, is_current, notes, created_at)
         VALUES (?1, ?2, 'intensive', ?3, ?4, ?5, ?6, 1, NULL, ?7)",
  )
  .bind(hn)
  .bind(regimen)
  .bind(start_date.format("%Y-%m-%d").to_string())
  .bind(intensive_end.format("%Y-%m-%d").to_string())
  .bind(HRZE)
  .bind(intensive_months as i64)
  .bind(now)
  .execute(&mut **tx)
  .await?;

  // ── Continuation phase (is_current = 0, pre-created) ─────────────────────
  if continuation_months > 0 {
    let cont_start = intensive_end;
    let cont_end = add_months(cont_start, continuation_months);

    sqlx::query(
      "INSERT INTO tb_treatment_plans
                 (hn, regimen, phase, phase_start, phase_end_expected,
                  drugs, duration_months, is_current, notes, created_at)
             VALUES (?1, ?2, 'continuation', ?3, ?4, ?5, ?6, 0, NULL, ?7)",
    )
    .bind(hn)
    .bind(regimen)
    .bind(cont_start.format("%Y-%m-%d").to_string())
    .bind(cont_end.format("%Y-%m-%d").to_string())
    .bind(HR)
    .bind(continuation_months as i64)
    .bind(now)
    .execute(&mut **tx)
    .await?;
  }

  Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// tb_treatment_plans — read
// ─────────────────────────────────────────────────────────────────────────────

/// Return the single plan with `is_current = 1` for this patient, or `None`.
pub async fn get_current_treatment_plan(
  pool: &SqlitePool,
  hn: &str,
) -> Result<Option<TreatmentPlan>> {
  sqlx::query_as::<_, TreatmentPlan>(
    "SELECT id, hn, regimen, phase, phase_start, phase_end_expected,
                drugs, duration_months, is_current, notes, created_at
         FROM   tb_treatment_plans
         WHERE  hn = ? AND is_current = 1
         ORDER  BY created_at DESC
         LIMIT  1",
  )
  .bind(hn)
  .fetch_optional(pool)
  .await
  .map_err(anyhow::Error::from)
}

/// Return all treatment plans for a patient in chronological order.
/// The alert engine sums `duration_months` across all rows to compute
/// the expected total treatment duration.
pub async fn get_all_treatment_plans(pool: &SqlitePool, hn: &str) -> Result<Vec<TreatmentPlan>> {
  sqlx::query_as::<_, TreatmentPlan>(
    "SELECT id, hn, regimen, phase, phase_start, phase_end_expected,
                drugs, duration_months, is_current, notes, created_at
         FROM   tb_treatment_plans
         WHERE  hn = ?
         ORDER  BY phase_start ASC, id ASC",
  )
  .bind(hn)
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

/// Return the earliest `phase_start` across all plans for `hn`, or `None`
/// when the patient has no plans yet.
///
/// `MIN()` always returns exactly one row; a `NULL` result (no plans) is
/// mapped to `Option::None` via `query_scalar`.
pub async fn get_first_phase_start(pool: &SqlitePool, hn: &str) -> Result<Option<String>> {
  let date: Option<String> =
    sqlx::query_scalar("SELECT MIN(phase_start) FROM tb_treatment_plans WHERE hn = ?")
      .bind(hn)
      .fetch_one(pool)
      .await?;
  Ok(date)
}

// ─────────────────────────────────────────────────────────────────────────────
// tb_treatment_plans — write
// ─────────────────────────────────────────────────────────────────────────────

/// Transition the patient to a new treatment phase within a single transaction:
///
/// 1. Mark every existing plan for this patient as `is_current = 0`.
/// 2. If a plan for the requested `new_phase` already exists (pre-created at
///    enrolment), **update** it in-place — this keeps `total_months` stable.
/// 3. Otherwise **insert** a brand-new plan row (handles custom / third-line
///    regimens that were not anticipated at enrolment).
pub async fn update_treatment_phase(pool: &SqlitePool, update: &TreatmentPlanUpdate) -> Result<()> {
  let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

  let phase_start = NaiveDate::parse_from_str(&update.phase_start, "%Y-%m-%d")
    .unwrap_or_else(|_| Local::now().date_naive());
  let phase_end = add_months(phase_start, update.duration_months as i32);
  let phase_end_str = phase_end.format("%Y-%m-%d").to_string();

  let mut tx = pool.begin().await?;

  // Step 1 — deactivate all current plans
  sqlx::query("UPDATE tb_treatment_plans SET is_current = 0 WHERE hn = ?")
    .bind(&update.hn)
    .execute(&mut *tx)
    .await?;

  // Step 2 — attempt in-place update of the matching phase row
  let rows = sqlx::query(
    "UPDATE tb_treatment_plans
         SET    is_current         = 1,
                regimen            = ?1,
                phase_start        = ?2,
                phase_end_expected = ?3,
                drugs              = ?4,
                duration_months    = ?5,
                notes              = ?6
         WHERE  hn = ?7 AND phase = ?8",
  )
  .bind(&update.regimen)
  .bind(&update.phase_start)
  .bind(&phase_end_str)
  .bind(&update.drugs)
  .bind(update.duration_months)
  .bind(&update.notes)
  .bind(&update.hn)
  .bind(&update.new_phase)
  .execute(&mut *tx)
  .await?
  .rows_affected();

  // Step 3 — fall back to insert when no matching row existed
  if rows == 0 {
    sqlx::query(
      "INSERT INTO tb_treatment_plans
                 (hn, regimen, phase, phase_start, phase_end_expected,
                  drugs, duration_months, is_current, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9)",
    )
    .bind(&update.hn)
    .bind(&update.regimen)
    .bind(&update.new_phase)
    .bind(&update.phase_start)
    .bind(&phase_end_str)
    .bind(&update.drugs)
    .bind(update.duration_months)
    .bind(&update.notes)
    .bind(&now)
    .execute(&mut *tx)
    .await?;
  }

  sqlx::query("UPDATE tb_patients SET updated_at = ?1 WHERE hn = ?2")
    .bind(&now)
    .bind(&update.hn)
    .execute(&mut *tx)
    .await?;

  tx.commit().await?;
  Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// tb_followups — read / write
// ─────────────────────────────────────────────────────────────────────────────

/// Insert a follow-up record and touch `updated_at` on the parent patient row,
/// both within a single transaction.
///
/// `FollowupInput.side_effects` is a `Vec<String>` on the Rust side; it is
/// serialised to a JSON array string (`"[\"nausea\",\"rash\"]"`) before
/// storage so the `Followup` model can return it as `Option<String>` directly.
pub async fn add_followup(pool: &SqlitePool, input: &FollowupInput) -> Result<i64> {
  let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

  let side_effects_json: Option<String> = input
    .side_effects
    .as_ref()
    .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()));

  let mut tx = pool.begin().await?;

  let id = sqlx::query(
    "INSERT INTO tb_followups
             (hn, followup_date, month_number, weight_kg, sputum_result,
              xray_result, side_effects, adherence, dispensed_drugs,
              notes, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10, ?11)",
  )
  .bind(&input.hn)
  .bind(&input.followup_date)
  .bind(input.month_number)
  .bind(input.weight_kg)
  .bind(&input.sputum_result)
  .bind(&input.xray_result)
  .bind(&side_effects_json)
  .bind(&input.adherence)
  .bind(&input.notes)
  .bind(&input.created_by)
  .bind(&now)
  .execute(&mut *tx)
  .await?
  .last_insert_rowid();

  sqlx::query("UPDATE tb_patients SET updated_at = ?1 WHERE hn = ?2")
    .bind(&now)
    .bind(&input.hn)
    .execute(&mut *tx)
    .await?;

  tx.commit().await?;
  Ok(id)
}

/// Return all follow-up records for `hn` in ascending chronological order.
pub async fn get_followups(pool: &SqlitePool, hn: &str) -> Result<Vec<Followup>> {
  sqlx::query_as::<_, Followup>(
    "SELECT id, hn, followup_date, month_number, weight_kg, sputum_result,
                xray_result, side_effects, adherence, dispensed_drugs,
                notes, created_by, created_at
         FROM   tb_followups
         WHERE  hn = ?
         ORDER  BY followup_date ASC, id ASC",
  )
  .bind(hn)
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

// ─────────────────────────────────────────────────────────────────────────────
// tb_outcomes — read / write
// ─────────────────────────────────────────────────────────────────────────────

/// Return the outcome record for `hn`, or `None` when the patient has not yet
/// been discharged.
pub async fn get_outcome(pool: &SqlitePool, hn: &str) -> Result<Option<Outcome>> {
  sqlx::query_as::<_, Outcome>(
    "SELECT id, hn, outcome, outcome_date, treatment_end, notes, created_by, created_at
         FROM   tb_outcomes
         WHERE  hn = ?",
  )
  .bind(hn)
  .fetch_optional(pool)
  .await
  .map_err(anyhow::Error::from)
}

/// Record a treatment outcome, set `tb_patients.status` to the corresponding
/// terminal state, and deactivate all treatment plans — all within one
/// transaction.
///
/// | `outcome` value        | `tb_patients.status` |
/// |------------------------|----------------------|
/// | cured / treatment_completed | completed       |
/// | died                   | died                 |
/// | lost_to_followup       | defaulted            |
/// | transferred_out        | transferred          |
/// | *(anything else)*      | completed            |
pub async fn discharge_patient(pool: &SqlitePool, input: &OutcomeInput) -> Result<()> {
  let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

  let patient_status = match input.outcome.as_str() {
    "cured" | "treatment_completed" => "completed",
    "died" => "died",
    "lost_to_followup" => "defaulted",
    "transferred_out" => "transferred",
    _ => "completed",
  };

  let mut tx = pool.begin().await?;

  // Upsert — the UNIQUE constraint on `hn` allows at most one outcome row
  sqlx::query(
    "INSERT INTO tb_outcomes
             (hn, outcome, outcome_date, treatment_end, notes, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(hn) DO UPDATE SET
             outcome       = excluded.outcome,
             outcome_date  = excluded.outcome_date,
             treatment_end = excluded.treatment_end,
             notes         = excluded.notes,
             created_by    = excluded.created_by",
  )
  .bind(&input.hn)
  .bind(&input.outcome)
  .bind(&input.outcome_date)
  .bind(&input.treatment_end)
  .bind(&input.notes)
  .bind(&input.created_by)
  .bind(&now)
  .execute(&mut *tx)
  .await?;

  sqlx::query("UPDATE tb_patients SET status = ?1, updated_at = ?2 WHERE hn = ?3")
    .bind(patient_status)
    .bind(&now)
    .bind(&input.hn)
    .execute(&mut *tx)
    .await?;

  // Deactivate all treatment plans — patient is no longer receiving treatment
  sqlx::query("UPDATE tb_treatment_plans SET is_current = 0 WHERE hn = ?")
    .bind(&input.hn)
    .execute(&mut *tx)
    .await?;

  tx.commit().await?;
  Ok(())
}
