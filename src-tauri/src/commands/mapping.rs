use crate::commands::settings::MySqlState;
use crate::db;
use crate::models::mapping::{BatchGeocodeResult, MappingPatientRow, MappingSummary};
use crate::models::patient::{PatientDemographics, TbPatient};
use chrono::Local;
use reqwest::Client;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use std::time::Duration;
use tauri::State;

const JITTER_RANGE_DEGREES: f64 = 0.005;

fn geocode_lock() -> &'static tokio::sync::Mutex<()> {
  static GEOCODE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
  GEOCODE_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

#[derive(Debug, Deserialize)]
struct NominatimResult {
  lat: String,
  lon: String,
}

#[tauri::command]
pub async fn get_mapping_patients(
  sqlite: State<'_, SqlitePool>,
  mysql: State<'_, MySqlState>,
) -> Result<Vec<MappingPatientRow>, String> {
  let patients = db::sqlite::get_all_tb_patients(&sqlite)
    .await
    .map_err(|e| e.to_string())?;
  let locations = db::sqlite::get_all_patient_locations(&sqlite)
    .await
    .map_err(|e| e.to_string())?;

  let hns = patients
    .iter()
    .map(|patient| patient.hn.clone())
    .collect::<Vec<_>>();
  let mysql_pool = mysql.lock().await.clone();
  let demographics = if let Some(pool) = mysql_pool {
    db::mysql::get_patient_demographics_by_hns(&pool, &hns)
      .await
      .map_err(|e| e.to_string())?
  } else {
    HashMap::new()
  };

  Ok(build_mapping_rows(&patients, &locations, &demographics))
}

#[tauri::command]
pub async fn get_mapping_summary(
  sqlite: State<'_, SqlitePool>,
  mysql: State<'_, MySqlState>,
) -> Result<MappingSummary, String> {
  let rows = get_mapping_patients(sqlite, mysql).await?;

  let total_patients = rows.len() as i64;
  let active_patients = rows.iter().filter(|row| row.tb_status == "active").count() as i64;
  let mapped_patients = rows
    .iter()
    .filter(|row| row.geocode_status == "success" && row.lat.is_some() && row.lng.is_some())
    .count() as i64;
  let missing_address_patients = rows.iter().filter(|row| !row.has_address).count() as i64;

  Ok(MappingSummary {
    total_patients,
    active_patients,
    mapped_patients,
    unmapped_patients: total_patients - mapped_patients,
    missing_address_patients,
  })
}

#[tauri::command]
pub async fn geocode_patient_address(
  sqlite: State<'_, SqlitePool>,
  mysql: State<'_, MySqlState>,
  hn: String,
) -> Result<MappingPatientRow, String> {
  let mysql_pool = mysql
    .lock()
    .await
    .clone()
    .ok_or_else(|| "ยังไม่ได้เชื่อมต่อ HOSxP จึงไม่สามารถอ่านที่อยู่เพื่อทำแผนที่ได้".to_string())?;

  let patient = db::sqlite::get_patient_by_hn(&sqlite, &hn)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("ไม่พบผู้ป่วย HN {}", hn))?;

  let demographics = db::mysql::get_patient_demographics(&mysql_pool, &hn)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("ไม่พบข้อมูลผู้ป่วย HN {} ใน HOSxP", hn))?;

  geocode_patient_core(&sqlite, &patient, &demographics)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn batch_geocode_patients(
  sqlite: State<'_, SqlitePool>,
  mysql: State<'_, MySqlState>,
  limit: Option<usize>,
) -> Result<BatchGeocodeResult, String> {
  let mysql_pool = mysql
    .lock()
    .await
    .clone()
    .ok_or_else(|| "ยังไม่ได้เชื่อมต่อ HOSxP จึงไม่สามารถอ่านที่อยู่เพื่อทำแผนที่ได้".to_string())?;

  let patients = db::sqlite::get_all_tb_patients(&sqlite)
    .await
    .map_err(|e| e.to_string())?;
  let hns = patients
    .iter()
    .map(|patient| patient.hn.clone())
    .collect::<Vec<_>>();
  let demographics = db::mysql::get_patient_demographics_by_hns(&mysql_pool, &hns)
    .await
    .map_err(|e| e.to_string())?;
  let locations = db::sqlite::get_all_patient_locations(&sqlite)
    .await
    .map_err(|e| e.to_string())?;

  let mut processed = 0_i64;
  let mut succeeded = 0_i64;
  let mut failed = 0_i64;
  let mut skipped = 0_i64;
  let capped_limit = limit.unwrap_or(25).clamp(1, 100);

  for patient in patients {
    if processed >= capped_limit as i64 {
      break;
    }

    let Some(demographics_row) = demographics.get(&patient.hn) else {
      skipped += 1;
      continue;
    };

    let has_cached_success = locations
      .get(&patient.hn)
      .map(|location| {
        location.geocode_status == "success"
          && location.lat.is_some()
          && location.lng.is_some()
          && normalize_address(&location.raw_address)
            == normalize_address_opt(demographics_row.address.as_deref())
      })
      .unwrap_or(false);

    if has_cached_success {
      skipped += 1;
      continue;
    }

    processed += 1;
    match geocode_patient_core(&sqlite, &patient, demographics_row).await {
      Ok(_) => succeeded += 1,
      Err(_) => failed += 1,
    }
  }

  Ok(BatchGeocodeResult {
    processed,
    succeeded,
    failed,
    skipped,
  })
}

fn build_mapping_rows(
  patients: &[TbPatient],
  locations: &HashMap<String, crate::models::mapping::TbPatientLocation>,
  demographics: &HashMap<String, PatientDemographics>,
) -> Vec<MappingPatientRow> {
  patients
    .iter()
    .map(|patient| {
      let demographics_row = demographics.get(&patient.hn);
      let location = locations.get(&patient.hn);
      let live_address = demographics_row.and_then(|row| row.address.as_deref());
      let cached_address = location.map(|item| item.raw_address.as_str());
      let effective_address = live_address.or(cached_address);
      let address_changed = match (live_address, location) {
        (Some(address), Some(location_row)) => {
          normalize_address(address) != normalize_address(&location_row.raw_address)
        }
        _ => false,
      };

      let geocode_status = if !has_text(effective_address) {
        "missing_address".to_string()
      } else if address_changed {
        "pending".to_string()
      } else {
        location
          .map(|item| item.geocode_status.clone())
          .unwrap_or_else(|| "pending".to_string())
      };

      MappingPatientRow {
        hn: patient.hn.clone(),
        masked_hn: mask_hn(&patient.hn),
        masked_name: demographics_row
          .map(|row| mask_name(&row.full_name))
          .unwrap_or_else(|| "ไม่ระบุชื่อ".to_string()),
        tb_status: patient.status.clone(),
        tb_type: patient.tb_type.clone(),
        enrolled_at: patient.enrolled_at.clone(),
        diagnosis_date: patient.diagnosis_date.clone(),
        has_address: has_text(effective_address),
        address_preview: effective_address.map(address_preview),
        geocode_status,
        geocode_error: if address_changed {
          None
        } else {
          location.and_then(|item| item.geocode_error.clone())
        },
        lat: if address_changed {
          None
        } else {
          location.and_then(|item| item.jittered_lat.or(item.lat))
        },
        lng: if address_changed {
          None
        } else {
          location.and_then(|item| item.jittered_lng.or(item.lng))
        },
        geocoded_at: location.and_then(|item| item.geocoded_at.clone()),
      }
    })
    .collect()
}

fn build_single_mapping_row(
  patient: &TbPatient,
  location: crate::models::mapping::TbPatientLocation,
  demographics: PatientDemographics,
) -> Result<MappingPatientRow, anyhow::Error> {
  build_mapping_rows(
    std::slice::from_ref(patient),
    &HashMap::from([(patient.hn.clone(), location)]),
    &HashMap::from([(patient.hn.clone(), demographics)]),
  )
  .into_iter()
  .next()
  .ok_or_else(|| anyhow::anyhow!("ไม่สามารถสร้างข้อมูลแผนที่ของผู้ป่วยได้"))
}

async fn geocode_patient_core(
  sqlite: &SqlitePool,
  patient: &TbPatient,
  demographics: &PatientDemographics,
) -> Result<MappingPatientRow, anyhow::Error> {
  let raw_address = demographics
    .address
    .as_deref()
    .filter(|address| has_text(Some(address)))
    .ok_or_else(|| anyhow::anyhow!("ผู้ป่วยรายนี้ไม่มีข้อมูลที่อยู่ใน HOSxP"))?
    .trim()
    .to_string();

  let normalized_address = normalize_address(&raw_address);
  let existing = db::sqlite::get_patient_location(sqlite, &patient.hn).await?;

  if let Some(location) = &existing {
    if location.geocode_status == "success"
      && location.lat.is_some()
      && location.lng.is_some()
      && location.normalized_address.as_deref() == Some(normalized_address.as_str())
    {
      return build_single_mapping_row(patient, location.clone(), demographics.clone());
    }
  }

  let previous_attempts = existing
    .as_ref()
    .map(|item| item.geocode_attempts)
    .unwrap_or(0);
  let geocode_result = geocode_address_with_rate_limit(&normalized_address).await;

  let (lat, lng, jittered_lat, jittered_lng, geocode_status, geocode_error, geocoded_at) =
    match geocode_result {
      Ok((lat, lng)) => {
        let (j_lat, j_lng) = jitter_coordinates(lat, lng, &patient.hn);
        (
          Some(lat),
          Some(lng),
          Some(j_lat),
          Some(j_lng),
          "success".to_string(),
          None,
          Some(Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()),
        )
      }
      Err(error) => (
        None,
        None,
        None,
        None,
        "failed".to_string(),
        Some(error.to_string()),
        None,
      ),
    };

  db::sqlite::upsert_patient_location(
    sqlite,
    &db::sqlite::UpsertPatientLocationInput {
      hn: patient.hn.clone(),
      raw_address: raw_address.clone(),
      normalized_address: Some(normalized_address),
      lat,
      lng,
      jittered_lat,
      jittered_lng,
      geocode_status,
      geocode_error,
      geocode_attempts: previous_attempts + 1,
      geocoded_at,
    },
  )
  .await?;

  let location = db::sqlite::get_patient_location(sqlite, &patient.hn)
    .await?
    .ok_or_else(|| anyhow::anyhow!("บันทึกตำแหน่งผู้ป่วยไม่สำเร็จ"))?;

  build_single_mapping_row(patient, location, demographics.clone())
}

async fn geocode_address_with_rate_limit(address: &str) -> Result<(f64, f64), anyhow::Error> {
  let _guard = geocode_lock().lock().await;
  let client = Client::builder()
    .timeout(Duration::from_secs(15))
    .user_agent("TBClinicMapping/1.0 (sabot-tb-clinic)")
    .build()?;

  let url = "https://nominatim.openstreetmap.org/search";
  let mut last_error: Option<anyhow::Error> = None;
  let candidates = build_geocode_queries(address);

  for candidate in candidates {
    for attempt in 0..3 {
      tokio::time::sleep(Duration::from_secs(1)).await;

      let url_with_params = reqwest::Url::parse_with_params(
        url,
        &[
          ("q", candidate.as_str()),
          ("format", "jsonv2"),
          ("limit", "1"),
          ("countrycodes", "th"),
        ],
      )?;
      let response = client.get(url_with_params).send().await;

      match response {
        Ok(result) => {
          let result = result.error_for_status()?;
          let rows = result.json::<Vec<NominatimResult>>().await?;
          if let Some(first) = rows.first() {
            let lat = first.lat.parse::<f64>()?;
            let lng = first.lon.parse::<f64>()?;
            return Ok((lat, lng));
          }
          last_error = Some(anyhow::anyhow!("ไม่พบพิกัดจากที่อยู่นี้"));
        }
        Err(error) => {
          last_error = Some(anyhow::anyhow!(error));
        }
      }

      if attempt < 2 {
        tokio::time::sleep(Duration::from_secs(1_u64 << attempt)).await;
      }
    }
  }

  Err(last_error.unwrap_or_else(|| anyhow::anyhow!("ไม่สามารถแปลงที่อยู่เป็นพิกัดได้")))
}

fn normalize_address(raw_address: &str) -> String {
  let collapsed = raw_address
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .replace(',', " ");
  let collapsed = collapsed
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .trim()
    .to_string();

  if collapsed.contains("ประเทศไทย") || collapsed.to_lowercase().contains("thailand")
  {
    collapsed
  } else if collapsed.is_empty() {
    collapsed
  } else {
    format!("{collapsed} ประเทศไทย")
  }
}

fn normalize_address_opt(raw_address: Option<&str>) -> String {
  raw_address.map(normalize_address).unwrap_or_default()
}

fn build_geocode_queries(address: &str) -> Vec<String> {
  let normalized = normalize_address(address);
  let raw_without_country = normalized
    .strip_suffix(" ประเทศไทย")
    .unwrap_or(&normalized)
    .trim()
    .to_string();
  let expanded = expand_thai_address_tokens(&raw_without_country);

  let mut queries = Vec::new();
  push_unique_query(&mut queries, normalized);

  if expanded != raw_without_country {
    push_unique_query(&mut queries, normalize_address(&expanded));
  }

  let subdistrict = extract_admin_segment(&expanded, &["ตำบล", "ต."]);
  let district = extract_admin_segment(&expanded, &["อำเภอ", "อ."]);
  let province = extract_admin_segment(&expanded, &["จังหวัด", "จ."]);

  if let (Some(subdistrict), Some(district), Some(province)) = (
    subdistrict.as_deref(),
    district.as_deref(),
    province.as_deref(),
  ) {
    push_unique_query(
      &mut queries,
      normalize_address(&format!("{subdistrict} {district} {province}")),
    );
    push_unique_query(
      &mut queries,
      normalize_address(&format!(
        "ตำบล{subdistrict} อำเภอ{district} จังหวัด{province}"
      )),
    );
  }

  if let (Some(district), Some(province)) = (district.as_deref(), province.as_deref()) {
    push_unique_query(
      &mut queries,
      normalize_address(&format!("{district} {province}")),
    );
    push_unique_query(
      &mut queries,
      normalize_address(&format!("อำเภอ{district} จังหวัด{province}")),
    );
  }

  if let Some(stripped) = strip_house_number_prefix(&expanded) {
    push_unique_query(&mut queries, normalize_address(&stripped));
  }

  queries
}

fn push_unique_query(queries: &mut Vec<String>, candidate: String) {
  let trimmed = candidate.trim();
  if trimmed.is_empty() {
    return;
  }

  if !queries.iter().any(|existing| existing == trimmed) {
    queries.push(trimmed.to_string());
  }
}

fn expand_thai_address_tokens(address: &str) -> String {
  address
    .replace("ต.", "ตำบล")
    .replace("อ.", "อำเภอ")
    .replace("จ.", "จังหวัด")
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn extract_admin_segment(address: &str, prefixes: &[&str]) -> Option<String> {
  for token in address.split_whitespace() {
    for prefix in prefixes {
      if token.starts_with(prefix) {
        let suffix = token.trim_start_matches(prefix).trim();
        if !suffix.is_empty() {
          return Some(suffix.to_string());
        }
      }
    }
  }

  None
}

fn strip_house_number_prefix(address: &str) -> Option<String> {
  let tokens = address.split_whitespace().collect::<Vec<_>>();
  let mut start_index = 0;

  while start_index < tokens.len() {
    let token = tokens[start_index];
    let is_house_number = token.chars().any(|ch| ch.is_ascii_digit())
      && !token.starts_with("ตำบล")
      && !token.starts_with("อำเภอ");
    let is_moo_marker = token == "หมู่" || token.starts_with("หมู่");

    if is_house_number || is_moo_marker {
      start_index += 1;
      continue;
    }

    break;
  }

  if start_index >= tokens.len() {
    None
  } else {
    Some(tokens[start_index..].join(" "))
  }
}

fn jitter_coordinates(lat: f64, lng: f64, key: &str) -> (f64, f64) {
  let lat_offset = deterministic_offset(&(key.to_string() + "-lat"));
  let lng_offset = deterministic_offset(&(key.to_string() + "-lng"));
  (
    (lat + lat_offset).clamp(-90.0, 90.0),
    (lng + lng_offset).clamp(-180.0, 180.0),
  )
}

fn deterministic_offset(key: &str) -> f64 {
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  key.hash(&mut hasher);
  let hash = hasher.finish();
  let normalized = (hash % 10_001) as f64 / 10_000.0;
  (normalized - 0.5) * JITTER_RANGE_DEGREES * 2.0
}

fn has_text(value: Option<&str>) -> bool {
  value.map(|text| !text.trim().is_empty()).unwrap_or(false)
}

fn mask_hn(hn: &str) -> String {
  let suffix = hn
    .chars()
    .rev()
    .take(4)
    .collect::<Vec<_>>()
    .into_iter()
    .rev()
    .collect::<String>();
  format!("HN ••••{suffix}")
}

fn mask_name(full_name: &str) -> String {
  let compact = full_name.trim();
  if compact.is_empty() {
    return "ไม่ระบุชื่อ".to_string();
  }

  let parts = compact.split_whitespace().collect::<Vec<_>>();
  if parts.len() >= 2 {
    let first = parts[0].chars().take(1).collect::<String>();
    let second = parts[1].chars().take(1).collect::<String>();
    return format!("{first}•• {second}••");
  }

  let head = compact.chars().take(2).collect::<String>();
  format!("{head}••")
}

fn address_preview(raw_address: &str) -> String {
  let compact = raw_address.split_whitespace().collect::<Vec<_>>().join(" ");
  if compact.chars().count() <= 40 {
    compact
  } else {
    format!("{}...", compact.chars().take(40).collect::<String>())
  }
}
