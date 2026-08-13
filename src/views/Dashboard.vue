<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAppStore } from "../stores/app";
import { getDashboardStats, getOverdueInstances, getTodayInstances, completeInstance, uncompleteInstance } from "../api";
import Pagination from "../components/Pagination.vue";
import { showError } from "../utils/error";

const store = useAppStore();
const todayList = ref<any[]>([]);
const overdueList = ref<any[]>([]);

const todayPage = ref(1);
const todayTotal = ref(0);
const todayPerPage = ref(10);

const overduePage = ref(1);
const overdueTotal = ref(0);
const overduePerPage = ref(10);

const getPageSize = () =>
  Math.max(5, Math.min(100, parseInt(localStorage.getItem("pageSize") || "10", 10) || 10));

const cycleLabel = (type: string) => {
  const map: Record<string, string> = {
    monthly: "每月",
    quarterly: "每季度",
    halfyearly: "每半年",
    yearly: "每年",
  };
  return map[type] || type;
};

const statusLabel = (status: string) => {
  const map: Record<string, string> = {
    pending: "待办",
    completed: "已完成",
    overdue: "逾期",
  };
  return map[status] || status;
};

const load = async () => {
  store.stats = await getDashboardStats();

  todayPerPage.value = getPageSize();
  const todayResult = await getTodayInstances(todayPage.value, todayPerPage.value);
  todayList.value = todayResult.items;
  todayTotal.value = todayResult.total;

  overduePerPage.value = getPageSize();
  const overdueResult = await getOverdueInstances(overduePage.value, overduePerPage.value);
  overdueList.value = overdueResult.items;
  overdueTotal.value = overdueResult.total;
};

const toggleComplete = async (item: any) => {
  try {
    if (item.instance.status === "completed") {
      await uncompleteInstance(item.instance.id);
    } else {
      await completeInstance(item.instance.id);
    }
    await load();
  } catch (e) {
    showError(e);
  }
};

const changeTodayPage = (page: number) => {
  todayPage.value = page;
  load();
};

const changeOverduePage = (page: number) => {
  overduePage.value = page;
  load();
};

const assigneeNames = (item: any) => {
  if (!item.assignees || item.assignees.length === 0) return "（未指派）";
  return item.assignees.map((m: any) => m.name).join("、");
};

onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <!-- Stats -->
    <div class="grid grid-cols-4 gap-4">
      <div class="bg-white p-6 rounded-xl shadow-sm border">
        <div class="text-sm text-slate-500">今日待办</div>
        <div class="text-3xl font-bold text-slate-800 mt-2">{{ store.stats.today_count }}</div>
      </div>
      <div class="bg-white p-6 rounded-xl shadow-sm border">
        <div class="text-sm text-slate-500">待办任务</div>
        <div class="text-3xl font-bold text-amber-600 mt-2">{{ store.stats.pending_count }}</div>
      </div>
      <div class="bg-white p-6 rounded-xl shadow-sm border">
        <div class="text-sm text-slate-500">逾期任务</div>
        <div class="text-3xl font-bold text-red-600 mt-2">{{ store.stats.overdue_count }}</div>
      </div>
      <div class="bg-white p-6 rounded-xl shadow-sm border">
        <div class="text-sm text-slate-500">已完成</div>
        <div class="text-3xl font-bold text-green-600 mt-2">{{ store.stats.completed_count }}</div>
      </div>
    </div>

    <!-- Today -->
    <div class="bg-white rounded-xl shadow-sm border overflow-hidden">
      <div class="px-6 py-4 border-b bg-slate-50">
        <h3 class="font-semibold text-slate-800">今日待办</h3>
      </div>
      <div class="divide-y">
        <div
          v-for="item in todayList"
          :key="item.instance.id"
          class="px-6 py-4 flex items-center justify-between hover:bg-slate-50"
        >
          <div>
            <div class="font-medium text-slate-800">{{ item.task.title }}</div>
            <div class="text-sm text-slate-500 mt-1">
              负责人：{{ assigneeNames(item) }} · 周期：{{ cycleLabel(item.task.cycle_type) }} ·
              截止：{{ item.instance.due_date }}
            </div>
          </div>
          <button
            @click="toggleComplete(item)"
            :class="[
              'px-4 py-2 text-sm rounded-lg transition',
              item.instance.status === 'completed'
                ? 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                : 'bg-blue-600 text-white hover:bg-blue-700',
            ]"
          >
            {{ item.instance.status === "completed" ? "撤销完成" : "完成" }}
          </button>
        </div>
        <div v-if="todayList.length === 0" class="px-6 py-8 text-center text-slate-400">
          今日没有待办任务
        </div>
      </div>
      <Pagination
        :page="todayPage"
        :per-page="todayPerPage"
        :total="todayTotal"
        @change="changeTodayPage"
      />
    </div>

    <!-- Overdue -->
    <div class="bg-white rounded-xl shadow-sm border overflow-hidden">
      <div class="px-6 py-4 border-b bg-slate-50">
        <h3 class="font-semibold text-slate-800">逾期任务</h3>
      </div>
      <div class="divide-y">
        <div
          v-for="item in overdueList"
          :key="item.instance.id"
          class="px-6 py-4 flex items-center justify-between hover:bg-slate-50"
        >
          <div>
            <div class="font-medium text-slate-800">{{ item.task.title }}</div>
            <div class="text-sm text-slate-500 mt-1">
              负责人：{{ assigneeNames(item) }} · 截止：{{ item.instance.due_date }}
            </div>
          </div>
          <div class="flex items-center gap-3">
            <span class="px-3 py-1 text-xs rounded-full bg-red-100 text-red-700">
              {{ statusLabel(item.instance.status) }}
            </span>
            <button
              @click="toggleComplete(item)"
              class="px-4 py-2 text-sm rounded-lg bg-green-600 text-white hover:bg-green-700 transition"
            >
              完成
            </button>
          </div>
        </div>
        <div v-if="overdueList.length === 0" class="px-6 py-8 text-center text-slate-400">
          没有逾期任务
        </div>
      </div>
      <Pagination
        :page="overduePage"
        :per-page="overduePerPage"
        :total="overdueTotal"
        @change="changeOverduePage"
      />
    </div>
  </div>
</template>
