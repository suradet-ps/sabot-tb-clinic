use crate::models::alert::PatientAlert;
use crate::models::dispensing::DispensingRecord;
use crate::models::treatment::{Followup, Outcome, TreatmentPlan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct TbPatient {
  pub id: i64,
  pub hn: String,
  pub enrolled_at: String,
  pub enrolled_by: Option<String>,
  pub status: String,
  pub tb_type: Option<String>,
  pub diagnosis_date: Option<String>,
  pub notes: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientDemographics {
  pub hn: String,
  pub full_name: String,
  pub age: Option<i64>,
  pub sex: Option<String>,
  pub address: Option<String>,
  pub phone: Option<String>,
  pub birthday: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientDrugRecord {
  pub hn: String,
  pub full_name: String,
  pub age: Option<i64>,
  pub sex: Option<String>,
  pub first_dispensed: Option<String>,
  pub last_dispensed: Option<String>,
  pub visit_count: i64,
  pub drug_names: Option<String>,
  pub drug_classes: Vec<String>,
  pub is_enrolled: bool,
  pub patient_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnrollmentInput {
  pub hn: String,
  pub tb_type: String,
  pub diagnosis_date: Option<String>,
  pub regimen: String,
  pub treatment_start_date: String,
  pub enrolled_by: Option<String>,
  pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientDetail {
  pub patient: TbPatient,
  pub demographics: Option<PatientDemographics>,
  pub current_plan: Option<TreatmentPlan>,
  pub followups: Vec<Followup>,
  pub outcome: Option<Outcome>,
  pub dispensing_history: Vec<DispensingRecord>,
  pub alerts: Vec<PatientAlert>,
  /// Whether the HOSxP MySQL connection was available when this detail was loaded.
  pub mysql_connected: bool,
  /// Error message from the MySQL side (demographics or dispensing fetch), if any.
  pub mysql_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivePatientRow {
  pub tb_patient: TbPatient,
  pub demographics: Option<PatientDemographics>,
  pub current_plan: Option<TreatmentPlan>,
  pub current_month: Option<i64>,
  pub total_months: Option<i64>,
  pub days_since_last_dispensing: Option<i64>,
  pub alerts: Vec<PatientAlert>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchFilters {
  pub date_from: Option<String>,
  pub date_to: Option<String>,
  pub drug_classes: Option<Vec<String>>,
  pub enrollment_status: Option<String>,
  pub hn_search: Option<String>,
  pub name_search: Option<String>,
  pub page: Option<i64>,
  pub page_size: Option<i64>,
}

/// A single TB clinic appointment record fetched from HOSxP `oapp` (clinic = '009').
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AppointmentRecord {
  pub hn: String,
  pub full_name: Option<String>,
  /// Appointment date as `YYYY-MM-DD`.
  pub nextdate: String,
}
