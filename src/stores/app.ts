import { defineStore } from "pinia";
import { ref } from "vue";

export interface Member {
  id: number;
  name: string;
  phone?: string;
  email?: string;
  note?: string;
  created_at: string;
}

export interface Task {
  id: number;
  title: string;
  description?: string;
  cycle_type: "monthly" | "quarterly" | "halfyearly" | "yearly";
  cycle_day: number;
  start_date: string;
  end_date?: string;
  reminder_minutes: number;
  sound_enabled: boolean;
  created_at: string;
}

export interface TaskWithAssignees {
  task: Task;
  assignees: Member[];
}

export interface TaskInstance {
  id: number;
  task_id: number;
  due_date: string;
  status: "pending" | "completed" | "overdue";
  confirmed_at?: string;
  reminded: boolean;
  created_at: string;
}

export interface TaskInstanceDetail {
  instance: TaskInstance;
  task: Task;
  assignees: Member[];
}

export interface DashboardStats {
  pending_count: number;
  overdue_count: number;
  completed_count: number;
  today_count: number;
}

export interface ArchiveCategory {
  id: number;
  name: string;
  code_prefix: string;
  note?: string;
  created_at: string;
}

export interface ArchiveTag {
  id: number;
  name: string;
  parent_id?: number;
  note?: string;
  created_at: string;
}

export interface ArchiveBox {
  id: number;
  name: string;
  location?: string;
  note?: string;
  created_at: string;
}

export interface ArchiveBoxSuggestion {
  box_name: string;
  reason: string;
  is_existing: boolean;
  matched_box_id?: number;
}

export interface Archive {
  id: number;
  code: string;
  title: string;
  category_id: number;
  location?: string;
  keeper_id?: number;
  status: "in_stock" | "borrowed" | "damaged" | "destroyed";
  quantity: number;
  description?: string;
  photos?: string;
  archive_box_id?: number;
  box_name?: string;
  file_path?: string;
  created_at: string;
}

export interface ArchiveDetail {
  archive: Archive;
  category?: ArchiveCategory;
  keeper?: Member;
  tags: ArchiveTag[];
}

export interface ArchiveBorrow {
  id: number;
  archive_id: number;
  borrower_id: number;
  purpose?: string;
  borrow_date: string;
  due_date: string;
  return_date?: string;
  status: "borrowed" | "returned" | "overdue";
  approver_id?: number;
  note?: string;
  reminded: boolean;
  created_at: string;
}

export interface ArchiveBorrowDetail {
  borrow: ArchiveBorrow;
  archive: ArchiveDetail;
  borrower: Member;
  approver?: Member;
}

export interface ArchiveStats {
  total_count: number;
  in_stock_count: number;
  borrowed_count: number;
  overdue_count: number;
  damaged_count: number;
  destroyed_count: number;
}

export interface Contract {
  id: number;
  contract_no?: string;
  contract_name: string;
  party_a?: string;
  party_b?: string;
  contact_person?: string;
  contact_info?: string;
  total_amount_with_tax?: number;
  total_amount_without_tax?: number;
  tax_amount?: number;
  payment_cycle?: string;
  payment_amount_with_tax?: number;
  payment_method?: string;
  effective_date?: string;
  end_date?: string;
  sign_date?: string;
  handler_party_a?: string;
  handler_party_b?: string;
  remark?: string;
  created_at: string;
}

export interface Paginated<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
}

export const useAppStore = defineStore("app", () => {
  const members = ref<Member[]>([]);
  const tasks = ref<TaskWithAssignees[]>([]);
  const instances = ref<TaskInstanceDetail[]>([]);
  const stats = ref<DashboardStats>({
    pending_count: 0,
    overdue_count: 0,
    completed_count: 0,
    today_count: 0,
  });
  const archiveCategories = ref<ArchiveCategory[]>([]);
  const archiveTags = ref<ArchiveTag[]>([]);
  const archiveBoxes = ref<ArchiveBox[]>([]);
  const archives = ref<ArchiveDetail[]>([]);
  const archiveBorrows = ref<ArchiveBorrowDetail[]>([]);
  const archiveStats = ref<ArchiveStats>({
    total_count: 0,
    in_stock_count: 0,
    borrowed_count: 0,
    overdue_count: 0,
    damaged_count: 0,
    destroyed_count: 0,
  });
  const contracts = ref<Contract[]>([]);

  return {
    members,
    tasks,
    instances,
    stats,
    archiveCategories,
    archiveTags,
    archiveBoxes,
    archives,
    archiveBorrows,
    archiveStats,
    contracts,
  };
});
