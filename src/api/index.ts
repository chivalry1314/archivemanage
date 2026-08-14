import { invoke } from "@tauri-apps/api/core";
import {
  type ArchiveBorrowDetail,
  type ArchiveBox,
  type ArchiveCategory,
  type ArchiveDetail,
  type ArchiveStats,
  type ArchiveTag,
  type DashboardStats,
  type Member,
  type Paginated,
  type TaskInstanceDetail,
  type TaskWithAssignees,
} from "../stores/app";

export interface CreateMemberRequest {
  name: string;
  phone?: string;
  email?: string;
  note?: string;
}

export interface UpdateMemberRequest {
  id: number;
  name: string;
  phone?: string;
  email?: string;
  note?: string;
}

export interface CreateTaskRequest {
  title: string;
  description?: string;
  cycle_type: "monthly" | "quarterly" | "halfyearly" | "yearly";
  cycle_day: number;
  start_date: string;
  end_date?: string;
  reminder_minutes: number;
  sound_enabled: boolean;
  assignee_ids: number[];
}

export interface UpdateTaskRequest {
  id: number;
  title: string;
  description?: string;
  cycle_type: "monthly" | "quarterly" | "halfyearly" | "yearly";
  cycle_day: number;
  start_date: string;
  end_date?: string;
  reminder_minutes: number;
  sound_enabled: boolean;
  assignee_ids: number[];
}

export const createMember = (req: CreateMemberRequest) =>
  invoke<Member>("create_member", { req });
export const updateMember = (req: UpdateMemberRequest) =>
  invoke<Member>("update_member", { req });
export const deleteMember = (id: number) => invoke<void>("delete_member", { id });
export const listMembers = () => invoke<Member[]>("list_members");
export const listMembersPaged = (page: number, perPage: number, search?: string) =>
  invoke<Paginated<Member>>("list_members_paged", { page, perPage, search });

export const createTask = (req: CreateTaskRequest) =>
  invoke<TaskWithAssignees>("create_task", { req });
export const updateTask = (req: UpdateTaskRequest) =>
  invoke<TaskWithAssignees>("update_task", { req });
export const deleteTask = (id: number) => invoke<void>("delete_task", { id });
export const listTasks = (page: number, perPage: number) =>
  invoke<Paginated<TaskWithAssignees>>("list_tasks", { page, perPage });

export const listInstances = (filter: string | undefined, page: number, perPage: number) =>
  invoke<Paginated<TaskInstanceDetail>>("list_instances", { filter, page, perPage });
export const listTaskInstances = (taskId: number, page: number, perPage: number) =>
  invoke<Paginated<TaskInstanceDetail>>("list_task_instances", { taskId, page, perPage });
export const getTodayInstances = (page: number, perPage: number) =>
  invoke<Paginated<TaskInstanceDetail>>("get_today_instances", { page, perPage });
export const getPendingInstances = (page: number, perPage: number) =>
  invoke<Paginated<TaskInstanceDetail>>("get_pending_instances", { page, perPage });
export const getOverdueInstances = (page: number, perPage: number) =>
  invoke<Paginated<TaskInstanceDetail>>("get_overdue_instances", { page, perPage });
export const completeInstance = (id: number) =>
  invoke<TaskInstanceDetail>("complete_instance", { id });
export const uncompleteInstance = (id: number) =>
  invoke<TaskInstanceDetail>("uncomplete_instance", { id });
export const getDashboardStats = () =>
  invoke<DashboardStats>("get_dashboard_stats");

export const exportInstancesCsv = () => invoke<string>("export_instances_csv");
export const exportInstancesJson = () => invoke<string>("export_instances_json");
export const exportMemberStatsCsv = () =>
  invoke<string>("export_member_stats_csv");
export const exportArchivesCsv = () => invoke<string>("export_archives_csv");
export const exportArchivesXlsx = () => invoke<number[]>("export_archives_xlsx");
export const exportArchiveBorrowsCsv = () =>
  invoke<string>("export_archive_borrows_csv");
export const exportArchiveBorrowsXlsx = () =>
  invoke<number[]>("export_archive_borrows_xlsx");
export const saveFile = (path: string, content: Uint8Array) =>
  invoke<void>("save_file_command", { path, content: Array.from(content) });
export const getDbPath = () => invoke<string>("get_db_path");

// AI
export interface AiConfig {
  enabled: boolean;
  base_url: string;
  model: string;
  api_key: string;
}

export interface AnalyzeArchiveBoxRequest {
  title: string;
  category_id?: number;
}

export interface ArchiveBoxSuggestion {
  box_name: string;
  reason: string;
  is_existing: boolean;
  matched_box_id?: number;
}

export const getAiConfig = () => invoke<AiConfig>("get_ai_config_command");
export const setAiConfig = (config: AiConfig) =>
  invoke<void>("set_ai_config_command", { config });
export const listAiModels = (config: AiConfig) =>
  invoke<string[]>("list_ai_models", { config });
export const analyzeArchiveBox = (
  req: AnalyzeArchiveBoxRequest,
  existingBoxes: ArchiveBox[]
) =>
  invoke<ArchiveBoxSuggestion>("analyze_archive_box", {
    req,
    existingBoxes,
  });
export const setDbPath = (path: string, migrate: boolean) =>
  invoke<string>("set_db_path_command", { path, migrate });

// Archive Boxes
export interface CreateArchiveBoxRequest {
  name: string;
  location?: string;
  note?: string;
}

export interface UpdateArchiveBoxRequest {
  id: number;
  name: string;
  location?: string;
  note?: string;
}

export const createArchiveBox = (req: CreateArchiveBoxRequest) =>
  invoke<ArchiveBox>("create_archive_box", { req });
export const updateArchiveBox = (req: UpdateArchiveBoxRequest) =>
  invoke<ArchiveBox>("update_archive_box", { req });
export const deleteArchiveBox = (id: number) => invoke<void>("delete_archive_box", { id });
export const listArchiveBoxes = () => invoke<ArchiveBox[]>("list_archive_boxes");
export const listArchiveBoxesPaged = (page: number, perPage: number, search?: string) =>
  invoke<Paginated<ArchiveBox>>("list_archive_boxes_paged", { page, perPage, search });
export const getArchiveBox = (id: number) => invoke<ArchiveBox>("get_archive_box", { id });

// Archive Categories
export interface CreateArchiveCategoryRequest {
  name: string;
  code_prefix: string;
  note?: string;
}

export interface UpdateArchiveCategoryRequest {
  id: number;
  name: string;
  code_prefix: string;
  note?: string;
}

export const createArchiveCategory = (req: CreateArchiveCategoryRequest) =>
  invoke<ArchiveCategory>("create_archive_category", { req });
export const updateArchiveCategory = (req: UpdateArchiveCategoryRequest) =>
  invoke<ArchiveCategory>("update_archive_category", { req });
export const deleteArchiveCategory = (id: number) =>
  invoke<void>("delete_archive_category", { id });
export const listArchiveCategories = () =>
  invoke<ArchiveCategory[]>("list_archive_categories");
export const listArchiveCategoriesPaged = (page: number, perPage: number, search?: string) =>
  invoke<Paginated<ArchiveCategory>>("list_archive_categories_paged", { page, perPage, search });

// Archive Tags
export interface CreateArchiveTagRequest {
  name: string;
  parent_id?: number;
  note?: string;
}

export interface UpdateArchiveTagRequest {
  id: number;
  name: string;
  parent_id?: number;
  note?: string;
}

export const createArchiveTag = (req: CreateArchiveTagRequest) =>
  invoke<ArchiveTag>("create_archive_tag", { req });
export const updateArchiveTag = (req: UpdateArchiveTagRequest) =>
  invoke<ArchiveTag>("update_archive_tag", { req });
export const deleteArchiveTag = (id: number) => invoke<void>("delete_archive_tag", { id });
export const listArchiveTags = () => invoke<ArchiveTag[]>("list_archive_tags");
export const listArchiveTagsPaged = (page: number, perPage: number, search?: string) =>
  invoke<Paginated<ArchiveTag>>("list_archive_tags_paged", { page, perPage, search });
export const listArchivesByTag = (tagId: number, page: number, perPage: number) =>
  invoke<Paginated<ArchiveDetail>>("list_archives_by_tag", { tagId, page, perPage });

// Archives
export interface CreateArchiveRequest {
  title: string;
  category_id: number;
  location?: string;
  keeper_id?: number;
  quantity: number;
  description?: string;
  photos?: string;
  archive_box_id?: number;
  box_name?: string;
  file_path?: string;
  source_file_path?: string;
  tag_ids: number[];
}

export interface UpdateArchiveRequest {
  id: number;
  title: string;
  category_id: number;
  location?: string;
  keeper_id?: number;
  quantity: number;
  description?: string;
  photos?: string;
  archive_box_id?: number;
  box_name?: string;
  file_path?: string;
  source_file_path?: string;
  tag_ids: number[];
}

export const createArchive = (req: CreateArchiveRequest) =>
  invoke<ArchiveDetail>("create_archive", { req });
export const updateArchive = (req: UpdateArchiveRequest) =>
  invoke<ArchiveDetail>("update_archive", { req });
export const deleteArchive = (id: number) => invoke<void>("delete_archive", { id });
export const listArchives = (
  categoryId: number | undefined,
  status: string | undefined,
  search: string | undefined,
  page: number,
  perPage: number
) =>
  invoke<Paginated<ArchiveDetail>>("list_archives", {
    categoryId,
    status,
    search,
    page,
    perPage,
  });
export const getArchive = (id: number) =>
  invoke<ArchiveDetail>("get_archive", { id });
export const updateArchiveStatus = (id: number, status: string) =>
  invoke<ArchiveDetail>("update_archive_status", { id, status });
export const getArchiveFilePath = (id: number) =>
  invoke<string>("get_archive_file_path", { id });

// Archive Borrows
export interface CreateArchiveBorrowRequest {
  archive_id: number;
  borrower_id: number;
  purpose?: string;
  borrow_date: string;
  due_date: string;
  approver_id?: number;
  note?: string;
}

export interface UpdateArchiveBorrowRequest {
  id: number;
  borrower_id: number;
  purpose?: string;
  borrow_date: string;
  due_date: string;
  return_date?: string;
  status: string;
  approver_id?: number;
  note?: string;
}

export const createArchiveBorrow = (req: CreateArchiveBorrowRequest) =>
  invoke<ArchiveBorrowDetail>("create_archive_borrow", { req });
export const returnArchiveBorrow = (id: number, return_date: string) =>
  invoke<ArchiveBorrowDetail>("return_archive_borrow", { id, returnDate: return_date });
export const updateArchiveBorrow = (req: UpdateArchiveBorrowRequest) =>
  invoke<ArchiveBorrowDetail>("update_archive_borrow", { req });
export const deleteArchiveBorrow = (id: number) =>
  invoke<void>("delete_archive_borrow", { id });
export const listArchiveBorrows = (
  status: string | undefined,
  archiveId: number | undefined,
  borrowerId: number | undefined,
  page: number,
  perPage: number
) =>
  invoke<Paginated<ArchiveBorrowDetail>>("list_archive_borrows", {
    status,
    archiveId,
    borrowerId,
    page,
    perPage,
  });
export const listActiveArchiveBorrows = (page: number, perPage: number) =>
  invoke<Paginated<ArchiveBorrowDetail>>("list_active_archive_borrows", { page, perPage });
export const getArchiveBorrow = (id: number) =>
  invoke<ArchiveBorrowDetail>("get_archive_borrow", { id });
export const getArchiveStats = () =>
  invoke<ArchiveStats>("get_archive_stats");
export const importArchivesFromExcel = (path: string) =>
  invoke<[number, number]>("import_archives_from_excel", { path });

// Mobile server
export interface ServerStatus {
  running: boolean;
  url?: string;
  port?: number;
  error?: string;
}

export const startMobileServer = (port: number) =>
  invoke<ServerStatus>("start_mobile_server", { port });
export const stopMobileServer = () =>
  invoke<ServerStatus>("stop_mobile_server");
export const getMobileServerStatus = () =>
  invoke<ServerStatus>("get_mobile_server_status");
