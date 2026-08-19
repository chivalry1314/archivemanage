use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: i64,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub cycle_type: String,
    pub cycle_day: i32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub reminder_minutes: i32,
    pub sound_enabled: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWithAssignees {
    #[serde(flatten)]
    pub task: Task,
    pub assignees: Vec<Member>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstance {
    pub id: i64,
    pub task_id: i64,
    pub due_date: NaiveDate,
    pub status: String,
    pub confirmed_at: Option<NaiveDateTime>,
    pub reminded: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstanceDetail {
    #[serde(flatten)]
    pub instance: TaskInstance,
    pub task: Task,
    pub assignees: Vec<Member>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub cycle_type: String,
    pub cycle_day: i32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub reminder_minutes: i32,
    pub sound_enabled: bool,
    pub assignee_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub cycle_type: String,
    pub cycle_day: i32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub reminder_minutes: i32,
    pub sound_enabled: bool,
    pub assignee_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemberRequest {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemberRequest {
    pub id: i64,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub pending_count: i64,
    pub overdue_count: i64,
    pub completed_count: i64,
    pub today_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveCategory {
    pub id: i64,
    pub name: String,
    pub code_prefix: String,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveBox {
    pub id: i64,
    pub name: String,
    pub location: Option<String>,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveTag {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archive {
    pub id: i64,
    pub code: String,
    pub title: String,
    pub category_id: i64,
    pub location: Option<String>,
    pub keeper_id: Option<i64>,
    pub status: String,
    pub quantity: i32,
    pub description: Option<String>,
    pub photos: Option<String>,
    pub archive_type: String,
    pub archive_box_id: Option<i64>,
    pub box_name: Option<String>,
    pub file_path: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveDetail {
    pub archive: Archive,
    pub category: Option<ArchiveCategory>,
    pub keeper: Option<Member>,
    pub tags: Vec<ArchiveTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveBorrow {
    pub id: i64,
    pub archive_id: i64,
    pub borrower_id: i64,
    pub purpose: Option<String>,
    pub borrow_date: NaiveDate,
    pub due_date: NaiveDate,
    pub return_date: Option<NaiveDate>,
    pub status: String,
    pub approver_id: Option<i64>,
    pub note: Option<String>,
    pub reminded: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveBorrowDetail {
    pub borrow: ArchiveBorrow,
    pub archive: ArchiveDetail,
    pub borrower: Member,
    pub approver: Option<Member>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArchiveCategoryRequest {
    pub name: String,
    pub code_prefix: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateArchiveCategoryRequest {
    pub id: i64,
    pub name: String,
    pub code_prefix: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArchiveRequest {
    pub title: String,
    pub category_id: i64,
    pub keeper_id: Option<i64>,
    pub quantity: i32,
    pub description: Option<String>,
    pub photos: Option<String>,
    pub archive_type: Option<String>,
    pub archive_box_id: Option<i64>,
    pub box_name: Option<String>,
    pub file_path: Option<String>,
    pub source_file_path: Option<String>,
    pub tag_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateArchiveRequest {
    pub id: i64,
    pub title: String,
    pub category_id: i64,
    pub keeper_id: Option<i64>,
    pub quantity: i32,
    pub description: Option<String>,
    pub photos: Option<String>,
    pub archive_type: Option<String>,
    pub archive_box_id: Option<i64>,
    pub box_name: Option<String>,
    pub file_path: Option<String>,
    pub source_file_path: Option<String>,
    pub tag_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArchiveBoxRequest {
    pub name: String,
    pub location: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateArchiveBoxRequest {
    pub id: i64,
    pub name: String,
    pub location: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://api.siliconflow.cn/v1".to_string(),
            model: "Qwen/Qwen2.5-7B-Instruct".to_string(),
            api_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeArchiveBoxRequest {
    pub title: String,
    pub category_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveBoxSuggestion {
    pub box_name: String,
    pub reason: String,
    pub is_existing: bool,
    pub matched_box_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArchiveTagRequest {
    pub name: String,
    pub parent_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateArchiveTagRequest {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArchiveBorrowRequest {
    pub archive_id: i64,
    pub borrower_id: i64,
    pub purpose: Option<String>,
    pub borrow_date: NaiveDate,
    pub due_date: NaiveDate,
    pub approver_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateArchiveBorrowRequest {
    pub id: i64,
    pub borrower_id: i64,
    pub purpose: Option<String>,
    pub borrow_date: NaiveDate,
    pub due_date: NaiveDate,
    pub return_date: Option<NaiveDate>,
    pub status: String,
    pub approver_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStats {
    pub total_count: i64,
    pub in_stock_count: i64,
    pub borrowed_count: i64,
    pub overdue_count: i64,
    pub damaged_count: i64,
    pub destroyed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: i64,
    pub contract_no: Option<String>,
    pub contract_name: String,
    pub party_a: Option<String>,
    pub party_b: Option<String>,
    pub contact_person: Option<String>,
    pub contact_info: Option<String>,
    pub total_amount_with_tax: Option<i64>,
    pub total_amount_without_tax: Option<i64>,
    pub tax_amount: Option<i64>,
    pub payment_cycle: Option<String>,
    pub payment_amount_with_tax: Option<i64>,
    pub payment_method: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub sign_date: Option<NaiveDate>,
    pub handler_party_a: Option<String>,
    pub handler_party_b: Option<String>,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContractRequest {
    pub contract_no: Option<String>,
    pub contract_name: String,
    pub party_a: Option<String>,
    pub party_b: Option<String>,
    pub contact_person: Option<String>,
    pub contact_info: Option<String>,
    pub total_amount_with_tax: Option<i64>,
    pub total_amount_without_tax: Option<i64>,
    pub tax_amount: Option<i64>,
    pub payment_cycle: Option<String>,
    pub payment_amount_with_tax: Option<i64>,
    pub payment_method: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub sign_date: Option<NaiveDate>,
    pub handler_party_a: Option<String>,
    pub handler_party_b: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContractRequest {
    pub id: i64,
    pub contract_no: Option<String>,
    pub contract_name: String,
    pub party_a: Option<String>,
    pub party_b: Option<String>,
    pub contact_person: Option<String>,
    pub contact_info: Option<String>,
    pub total_amount_with_tax: Option<i64>,
    pub total_amount_without_tax: Option<i64>,
    pub tax_amount: Option<i64>,
    pub payment_cycle: Option<String>,
    pub payment_amount_with_tax: Option<i64>,
    pub payment_method: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub sign_date: Option<NaiveDate>,
    pub handler_party_a: Option<String>,
    pub handler_party_b: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
