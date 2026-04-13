use crate::models::dispensing::DispensingRecord;
use crate::models::patient::{
  AppointmentRecord, PatientDemographics, PatientDrugRecord, SearchFilters,
};
use anyhow::Result;
use sqlx::MySqlPool;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// TB drug icode constants (Sarabosot Hospital)
// ---------------------------------------------------------------------------

const H_ICODES: &[&str] = &["1430104"];
const R_ICODES: &[&str] = &["1000265", "1000264"];
const E_ICODES: &[&str] = &["1600004", "1000129"];
const Z_ICODES: &[&str] = &["1000258"];

const ALL_TB_ICODES: &[&str] = &[
  "1430104", "1000265", "1000264", "1600004", "1000129", "1000258",
];

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn icodes_for_classes(classes: &[String]) -> Vec<&'static str> {
  let mut out = Vec::new();
  for c in classes {
    match c.to_uppercase().as_str() {
      "H" => out.extend_from_slice(H_ICODES),
      "R" => out.extend_from_slice(R_ICODES),
      "E" => out.extend_from_slice(E_ICODES),
      "Z" => out.extend_from_slice(Z_ICODES),
      _ => {}
    }
  }
  out
}

fn icode_to_class(icode: &str) -> Option<&'static str> {
  match icode {
    "1430104" => Some("H"),
    "1000265" | "1000264" => Some("R"),
    "1600004" | "1000129" => Some("E"),
    "1000258" => Some("Z"),
    _ => None,
  }
}

/// Convert a comma-separated icode list (from GROUP_CONCAT) into sorted,
/// de-duplicated drug class letters.
fn drug_classes_from_icode_csv(csv: &str) -> Vec<String> {
  let mut seen = Vec::<&'static str>::new();
  for code in csv.split(',') {
    if let Some(cls) = icode_to_class(code.trim()) {
      if !seen.contains(&cls) {
        seen.push(cls);
      }
    }
  }
  seen.into_iter().map(String::from).collect()
}

/// Build a `?, ?, …` placeholder string for an IN clause.
fn in_placeholders(count: usize) -> String {
  (0..count).map(|_| "?").collect::<Vec<_>>().join(", ")
}

// ---------------------------------------------------------------------------
// Intermediate FromRow structs (never serialised to frontend)
// ---------------------------------------------------------------------------

/// Raw row returned by the screening aggregate query.
#[derive(sqlx::FromRow)]
struct ScreeningRow {
  hn: String,
  full_name: String,
  age: Option<i64>,
  sex: Option<String>,
  first_dispensed: Option<String>,
  last_dispensed: Option<String>,
  visit_count: i64,
  /// Comma-separated raw icodes, e.g. "1000258,1000265,1430104"
  icode_list: Option<String>,
  drug_names: Option<String>,
}

/// Raw row returned by the patient demographics lookup.
#[derive(sqlx::FromRow)]
struct DemographicsRow {
  hn: String,
  full_name: String,
  age: Option<i64>,
  sex: Option<String>,
  address: Option<String>,
  phone: Option<String>,
  birthday: Option<String>,
}

// ---------------------------------------------------------------------------
// Public query functions
// ---------------------------------------------------------------------------

/// Search all patients who have ever received any of the 6 TB drug icodes.
///
/// - Optional date-range filter applied on `opitemrece.vstdate`.
/// - Optional drug-class multi-select filter reduces the IN clause.
/// - Optional `hn_search` / `name_search` filters applied as SQL WHERE predicates.
/// - Enrollment-status filter (`"all"` / `"enrolled"` / `"not_enrolled"` / `"discharged"`)
///   is applied in Rust using the pre-fetched `enrolled_map` (hn → status) so that
///   we never need to join against the local SQLite DB inside HOSxP.
/// - Results are paginated; default page size is 50, maximum is 200.
pub async fn search_tb_patients(
  pool: &MySqlPool,
  filters: &SearchFilters,
  enrolled_map: &HashMap<String, String>,
) -> Result<Vec<PatientDrugRecord>> {
  // Resolve which icodes to include based on the drug-class filter
  let icodes: Vec<&str> = filters
    .drug_classes
    .as_deref()
    .map(|classes| {
      let v = icodes_for_classes(classes);
      if v.is_empty() {
        ALL_TB_ICODES.to_vec()
      } else {
        v
      }
    })
    .unwrap_or_else(|| ALL_TB_ICODES.to_vec());

  // Build SQL dynamically — IN clause, optional date predicates, optional
  // hn/name search predicates all go into WHERE (before GROUP BY).
  let mut sql = format!(
    "SELECT \
            p.hn, \
            CONCAT(COALESCE(p.pname, ''), p.fname, ' ', p.lname) AS full_name, \
            TIMESTAMPDIFF(YEAR, p.birthday, CURDATE()) AS age, \
            p.sex, \
            DATE_FORMAT(MIN(o.vstdate), '%Y-%m-%d') AS first_dispensed, \
            DATE_FORMAT(MAX(o.vstdate), '%Y-%m-%d') AS last_dispensed, \
            COUNT(DISTINCT DATE_FORMAT(o.vstdate, '%Y-%m-%d')) AS visit_count, \
            GROUP_CONCAT(DISTINCT o.icode ORDER BY o.icode SEPARATOR ',') AS icode_list, \
            GROUP_CONCAT(DISTINCT d.name ORDER BY d.name SEPARATOR ', ') AS drug_names \
        FROM opitemrece o \
        JOIN patient p ON o.hn = p.hn \
        JOIN drugitems d ON o.icode = d.icode \
        WHERE o.icode IN ({}) ",
    in_placeholders(icodes.len())
  );

  if filters.date_from.is_some() {
    sql.push_str("AND o.vstdate >= ? ");
  }
  if filters.date_to.is_some() {
    sql.push_str("AND o.vstdate <= ? ");
  }
  // HN search — exact prefix match on the real column (before GROUP BY)
  if filters.hn_search.is_some() {
    sql.push_str("AND p.hn LIKE ? ");
  }
  // Name search — match against the concatenated name expression (before GROUP BY)
  if filters.name_search.is_some() {
    sql.push_str("AND CONCAT(COALESCE(p.pname,''), p.fname, ' ', p.lname) LIKE ? ");
  }

  sql.push_str("GROUP BY p.hn ORDER BY last_dispensed DESC");

  // Pagination
  let page = filters.page.unwrap_or(1).max(1);
  let page_size = filters.page_size.unwrap_or(50).clamp(1, 200);
  let offset = (page - 1) * page_size;
  sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

  // Bind parameters in the same order they appear in the SQL
  let mut q = sqlx::query_as::<_, ScreeningRow>(&sql);
  for icode in &icodes {
    q = q.bind(*icode);
  }
  if let Some(df) = &filters.date_from {
    q = q.bind(df.as_str());
  }
  if let Some(dt) = &filters.date_to {
    q = q.bind(dt.as_str());
  }
  if let Some(hn) = &filters.hn_search {
    q = q.bind(format!("{}%", hn));
  }
  if let Some(name) = &filters.name_search {
    q = q.bind(format!("%{}%", name));
  }

  let rows = q.fetch_all(pool).await?;

  let mut out = Vec::with_capacity(rows.len());
  for row in rows {
    let patient_status = enrolled_map.get(&row.hn).cloned();
    let is_enrolled = patient_status.is_some();

    // Apply enrollment-status filter in Rust (avoids a cross-DB join)
    match filters.enrollment_status.as_deref() {
      Some("enrolled") => match &patient_status {
        Some(s) if s == "active" => {}
        _ => continue,
      },
      Some("not_enrolled") if is_enrolled => continue,
      Some("discharged") => match &patient_status {
        Some(s) if s != "active" => {}
        _ => continue,
      },
      _ => {}
    }

    let drug_classes = row
      .icode_list
      .as_deref()
      .map(drug_classes_from_icode_csv)
      .unwrap_or_default();

    out.push(PatientDrugRecord {
      hn: row.hn,
      full_name: row.full_name,
      age: row.age,
      sex: row.sex,
      first_dispensed: row.first_dispensed,
      last_dispensed: row.last_dispensed,
      visit_count: row.visit_count,
      drug_names: row.drug_names,
      drug_classes,
      is_enrolled,
      patient_status,
    });
  }

  Ok(out)
}

/// Fetch HOSxP patient demographics for a single HN.
/// Returns `None` when the HN does not exist in the HOSxP `patient` table.
pub async fn get_patient_demographics(
  pool: &MySqlPool,
  hn: &str,
) -> Result<Option<PatientDemographics>> {
  let row = sqlx::query_as::<_, DemographicsRow>(
    "SELECT \
            p.hn, \
            CONCAT(COALESCE(p.pname, ''), p.fname, ' ', p.lname) AS full_name, \
            TIMESTAMPDIFF(YEAR, p.birthday, CURDATE()) AS age, \
            p.sex, \
            p.informaddr AS address, \
            p.hometel AS phone, \
            DATE_FORMAT(p.birthday, '%Y-%m-%d') AS birthday \
        FROM patient p \
        WHERE p.hn = ?",
  )
  .bind(hn)
  .fetch_optional(pool)
  .await?;

  Ok(row.map(|r| PatientDemographics {
    hn: r.hn,
    full_name: r.full_name,
    age: r.age,
    sex: r.sex,
    address: r.address,
    phone: r.phone,
    birthday: r.birthday,
  }))
}

/// Fetch every TB drug dispensing record for one HN from `opitemrece`, newest
/// first.  Drug class (H / R / E / Z) is resolved inline via a SQL CASE
/// expression so that the result maps directly onto `DispensingRecord`.
///
/// Design notes:
/// - Uses `WHERE o.hn = ?` directly (no patient JOIN) — avoids any HN
///   format mismatch between tables and keeps the query simple.
/// - LEFT JOIN drugitems so that rows are returned even when an icode does
///   not have a matching entry in drugitems; drug_name falls back to the
///   raw icode via COALESCE in that case.
/// - `CAST(o.qty AS DOUBLE)` avoids sqlx DECIMAL→f64 mapping issues that
///   silently kill the entire result set with unwrap_or_default().
pub async fn get_dispensing_history(pool: &MySqlPool, hn: &str) -> Result<Vec<DispensingRecord>> {
  sqlx::query_as::<_, DispensingRecord>(
    "SELECT \
            DATE_FORMAT(o.vstdate, '%Y-%m-%d') AS vstdate, \
            o.icode, \
            TRIM(CONCAT_WS(' ', COALESCE(d.name, o.icode), d.strength)) AS drug_name, \
            CAST(o.qty AS DOUBLE) AS qty, \
            d.units, \
            CASE o.icode \
                WHEN '1430104' THEN 'H' \
                WHEN '1000265' THEN 'R' \
                WHEN '1000264' THEN 'R' \
                WHEN '1600004' THEN 'E' \
                WHEN '1000129' THEN 'E' \
                WHEN '1000258' THEN 'Z' \
                ELSE NULL \
            END AS drug_class \
        FROM opitemrece o \
        LEFT JOIN drugitems d ON o.icode = d.icode \
        WHERE o.hn = ? \
          AND o.icode IN ('1430104','1000265','1000264','1600004','1000129','1000258') \
        ORDER BY o.vstdate DESC, o.icode",
  )
  .bind(hn)
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}

/// Return the most recent date on which any TB drug was dispensed to `hn` as
/// `YYYY-MM-DD`, or `None` when no dispensing records exist.
pub async fn get_last_dispensing_date(pool: &MySqlPool, hn: &str) -> Result<Option<String>> {
  // MAX() on an empty set returns a single row with a NULL value — fetch_one
  // is therefore safe here; query_scalar decodes the nullable result as
  // Option<String>.
  let date: Option<String> = sqlx::query_scalar(
    "SELECT DATE_FORMAT(MAX(vstdate), '%Y-%m-%d') \
         FROM opitemrece \
         WHERE hn = ? \
           AND icode IN ('1430104','1000265','1000264','1600004','1000129','1000258')",
  )
  .bind(hn)
  .fetch_one(pool)
  .await?;

  Ok(date)
}

/// Return `true` if Ethambutol (icode 1600004 **or** 1000129) was dispensed
/// to `hn` within the last `days` calendar days.
///
/// Used by the alert engine to detect Ethambutol-overrun during the
/// continuation phase.
pub async fn was_ethambutol_dispensed_recently(
  pool: &MySqlPool,
  hn: &str,
  days: i64,
) -> Result<bool> {
  let count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) \
         FROM opitemrece \
         WHERE hn = ? \
           AND icode IN ('1600004','1000129') \
           AND vstdate >= CURDATE() - INTERVAL ? DAY",
  )
  .bind(hn)
  .bind(days)
  .fetch_one(pool)
  .await?;

  Ok(count > 0)
}

/// Fetch upcoming TB clinic appointments from HOSxP `oapp` (clinic code `009`).
///
/// Returns every appointment whose `nextdate` falls between today and
/// `today + days_ahead` days (inclusive), ordered by date ascending.
///
/// Join with `patient` to provide the human-readable name.
pub async fn get_tb_appointments(
  pool: &MySqlPool,
  days_ahead: i64,
) -> Result<Vec<AppointmentRecord>> {
  sqlx::query_as::<_, AppointmentRecord>(
    "SELECT \
            a.hn, \
            CONCAT(COALESCE(p.pname, ''), p.fname, ' ', p.lname) AS full_name, \
            DATE_FORMAT(a.nextdate, '%Y-%m-%d') AS nextdate \
        FROM oapp a \
        JOIN patient p ON a.hn = p.hn \
        WHERE a.clinic = '009' \
          AND a.nextdate BETWEEN CURDATE() AND DATE_ADD(CURDATE(), INTERVAL ? DAY) \
        ORDER BY a.nextdate ASC",
  )
  .bind(days_ahead)
  .fetch_all(pool)
  .await
  .map_err(anyhow::Error::from)
}
