<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRoute } from "vue-router";
import { getDashboardStats, listMembers, listTasks, getArchiveStats, listArchives, listArchiveBorrows, listArchiveTags } from "./api";
import { useAppStore } from "./stores/app";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";

const store = useAppStore();
const route = useRoute();
const appVersion = ref("");

const navItems = [
  { path: "/", label: "仪表盘" },
  { path: "/tasks", label: "任务管理" },
  { path: "/members", label: "人员管理" },
  { path: "/archives", label: "档案管理" },
  { path: "/archive-tags", label: "档案标签" },
  { path: "/archive-categories", label: "档案分类" },
  { path: "/archive-boxes", label: "档案盒维护" },
  { path: "/settings", label: "设置" },
];

const isActive = (path: string) => route.path === path;

const getPageSize = () =>
  Math.max(5, Math.min(100, parseInt(localStorage.getItem("pageSize") || "10", 10) || 10));

const refreshData = async () => {
  try {
    const perPage = getPageSize();
    store.members = await listMembers();
    store.tasks = (await listTasks(1, perPage)).items;
    store.stats = await getDashboardStats();
    store.archiveStats = await getArchiveStats();
    store.archiveTags = await listArchiveTags();
    store.archives = (await listArchives(undefined, undefined, undefined, 1, perPage)).items;
    store.archiveBorrows = (await listArchiveBorrows(undefined, undefined, undefined, 1, perPage)).items;
  } catch (e) {
    console.error(e);
  }
};

const playBeep = () => {
  try {
    const AudioContext = (window as any).AudioContext || (window as any).webkitAudioContext;
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = "sine";
    osc.frequency.value = 880;
    gain.gain.setValueAtTime(0.3, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.5);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start();
    osc.stop(ctx.currentTime + 0.5);
  } catch {
    // ignore
  }
};

onMounted(async () => {
  await refreshData();

  try {
    appVersion.value = await getVersion();
  } catch {
    // 浏览器预览等非 Tauri 环境忽略
  }

  listen("archivemanage", (event) => {
    const payload = event.payload as any;
    if (payload.sound_enabled) {
      playBeep();
    }
  });

  listen("archive-reminder", () => {
    playBeep();
  });
});
</script>

<template>
  <div class="h-screen flex overflow-hidden">
    <!-- Sidebar -->
    <aside class="w-56 bg-slate-900 text-white flex flex-col overflow-y-auto">
      <div class="p-6">
        <h1 class="text-xl font-bold">档案管理OS</h1>
        <p class="text-slate-400 text-sm mt-1">任务提醒 + 物业档案管理</p>
      </div>

      <nav class="flex-1 px-4 space-y-2">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          :class="[
            'flex items-center px-4 py-3 rounded-lg transition',
            isActive(item.path)
              ? 'bg-blue-600 text-white'
              : 'text-slate-300 hover:bg-slate-800',
          ]"
        >
          {{ item.label }}
        </router-link>
      </nav>

      <div class="p-4 text-xs text-slate-500">v{{ appVersion || "0.1.0" }}</div>
    </aside>

    <!-- Main -->
    <main class="flex-1 flex flex-col bg-slate-50 overflow-hidden">
      <header class="bg-white border-b px-8 py-4 flex items-center">
        <h2 class="text-lg font-semibold text-slate-800">{{ route.name }}</h2>
      </header>

      <div class="flex-1 p-8 overflow-auto">
        <router-view />
      </div>
    </main>
  </div>
</template>

<style scoped></style>
