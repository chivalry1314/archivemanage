import { createRouter, createWebHashHistory } from "vue-router";
import Dashboard from "../views/Dashboard.vue";
import Tasks from "../views/Tasks.vue";
import Members from "../views/Members.vue";
import Settings from "../views/Settings.vue";
import Archives from "../views/Archives.vue";
import ArchiveCategories from "../views/ArchiveCategories.vue";
import ArchiveTags from "../views/ArchiveTags.vue";
import ArchiveBoxes from "../views/ArchiveBoxes.vue";
import Contracts from "../views/Contracts.vue";

const routes = [
  { path: "/", name: "仪表盘", component: Dashboard },
  { path: "/tasks", name: "任务管理", component: Tasks },
  { path: "/members", name: "人员管理", component: Members },
  { path: "/archives", name: "档案管理", component: Archives },
  { path: "/archive-tags", name: "档案标签", component: ArchiveTags },
  { path: "/archive-categories", name: "档案分类", component: ArchiveCategories },
  { path: "/archive-boxes", name: "档案盒维护", component: ArchiveBoxes },
  { path: "/contracts", name: "合同管理", component: Contracts },
  { path: "/settings", name: "设置", component: Settings },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
