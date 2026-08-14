const { chromium } = require('playwright');
const fs = require('fs');

const BASE_URL = 'http://127.0.0.1:3456';
const OUTPUT_DIR = 'docs/public/images';

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const members = [
  { id: 1, name: '张三', phone: '13800138001', email: 'zhangsan@example.com', note: '物业主管', created_at: '2026-01-10 09:00:00' },
  { id: 2, name: '李四', phone: '13800138002', email: 'lisi@example.com', note: '档案管理员', created_at: '2026-01-12 10:30:00' },
];

const categories = [
  { id: 1, name: '业主档案', code_prefix: 'YZ', note: '业主相关资料', created_at: '2026-01-10 09:00:00' },
  { id: 2, name: '设备档案', code_prefix: 'SB', note: '设备维保资料', created_at: '2026-01-11 10:00:00' },
];

const tags = [
  { id: 1, name: '重要合同', parent_id: null, note: '', created_at: '2026-01-10 09:00:00' },
  { id: 2, name: '2026年度', parent_id: null, note: '', created_at: '2026-01-10 09:00:00' },
];

const tasks = [
  {
    task: { id: 1, title: '月度消防检查', description: '检查消防设施状态', cycle_type: 'monthly', cycle_day: 15, start_date: '2026-01-01', end_date: '', reminder_minutes: 30, sound_enabled: true },
    assignees: [{ id: 1, name: '张三' }],
  },
];

const taskInstances = [
  { instance: { id: 1, task_id: 1, due_date: '2026-08-15', status: 'pending' }, task: { title: '月度消防检查' }, assignees: [{ name: '张三' }] },
  { instance: { id: 2, task_id: 1, due_date: '2026-07-15', status: 'completed' }, task: { title: '月度消防检查' }, assignees: [{ name: '张三' }] },
  { instance: { id: 3, task_id: 1, due_date: '2026-06-15', status: 'overdue' }, task: { title: '月度消防检查' }, assignees: [{ name: '张三' }] },
];

const archives = [
  {
    archive: { id: 1, code: 'YZ00001', title: '1号楼业主资料', category_id: 1, location: '档案室-A柜-1层', box_name: '1号楼业主盒', file_path: '', source_file_path: '', keeper_id: 2, quantity: 1, description: '2026年度业主资料', photos: '', status: 'in_stock' },
    category: { name: '业主档案' },
    keeper: { name: '李四' },
    tags: [{ id: 1, name: '重要合同' }],
  },
];

const borrows = [
  { borrow: { id: 1, archive_id: 1, borrower_id: 1, purpose: '年度审计', borrow_date: '2026-08-01', due_date: '2026-08-20', return_date: '', status: 'borrowed', approver_id: 2, note: '' }, archive: { archive: { code: 'YZ00001', title: '1号楼业主资料' } }, borrower: { name: '张三' }, approver: { name: '李四' } },
];

async function takeScreenshots() {
  const browser = await chromium.launch({
    headless: true,
    executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe',
  });
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });

  await page.addInitScript((mockData) => {
    const { members, categories, tags, tasks, taskInstances, archives, borrows } = mockData;

    window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
    window.__TAURI_INTERNALS__.invoke = (cmd, args) => {
      const returnValue = (data) => Promise.resolve(data);

      switch (cmd) {
        case 'get_version':
        case 'get_app_version':
          return returnValue('0.1.0');
        case 'get_dashboard_stats':
          return returnValue({ today_count: 1, pending_count: 3, overdue_count: 1, completed_count: 5 });
        case 'get_today_instances':
          return returnValue({ items: [taskInstances[0]], total: 1 });
        case 'get_overdue_instances':
          return returnValue({ items: [taskInstances[2]], total: 1 });
        case 'list_tasks':
          return returnValue({ items: tasks, total: tasks.length });
        case 'list_task_instances':
          return returnValue({ items: taskInstances, total: taskInstances.length });
        case 'list_members':
          return returnValue(members);
        case 'list_members_paged':
          return returnValue({ items: members, total: members.length });
        case 'list_archive_categories':
          return returnValue(categories);
        case 'list_archive_categories_paged':
          return returnValue({ items: categories, total: categories.length });
        case 'list_archive_tags':
          return returnValue(tags);
        case 'list_archive_tags_paged':
          return returnValue({ items: tags, total: tags.length });
        case 'list_archives':
          return returnValue({ items: archives, total: archives.length });
        case 'list_archive_borrows':
          return returnValue({ items: borrows, total: borrows.length });
        case 'get_archive_stats':
          return returnValue({ total_count: 1, in_stock_count: 1, borrowed_count: 0, damaged_count: 0, destroyed_count: 0, overdue_count: 0 });
        case 'get_db_path':
          return returnValue('C:/Users/example/AppData/Roaming/task_reminder.db');
        case 'get_mobile_server_status':
          return returnValue({ running: false });
        default:
          return returnValue({});
      }
    };

    window.__TAURI__ = window.__TAURI__ || {};
    window.__TAURI__.event = { listen: () => Promise.resolve(() => {}) };
    window.__TAURI__.app = { getVersion: () => Promise.resolve('0.1.0') };
  }, { members, categories, tags, tasks, taskInstances, archives, borrows });

  // Dashboard
  await page.goto(`${BASE_URL}/#/`, { waitUntil: 'networkidle', timeout: 10000 });
  await sleep(1500);
  await page.screenshot({ path: `${OUTPUT_DIR}/dashboard.png`, fullPage: false });
  fs.copyFileSync(`${OUTPUT_DIR}/dashboard.png`, `${OUTPUT_DIR}/main-layout.png`);
  console.log('Captured dashboard.png / main-layout.png');

  // Tasks list
  await page.goto(`${BASE_URL}/#/tasks`, { waitUntil: 'networkidle', timeout: 10000 });
  await sleep(1000);
  await page.screenshot({ path: `${OUTPUT_DIR}/tasks-list.png`, fullPage: false });
  console.log('Captured tasks-list.png');

  // Task instances modal
  await page.click('button:has-text("实例")');
  await sleep(800);
  await page.screenshot({ path: `${OUTPUT_DIR}/task-instances.png`, fullPage: false });
  console.log('Captured task-instances.png');
  await page.keyboard.press('Escape');
  await sleep(300);

  // Members
  await page.goto(`${BASE_URL}/#/members`, { waitUntil: 'networkidle', timeout: 10000 });
  await sleep(1000);
  await page.screenshot({ path: `${OUTPUT_DIR}/members-list.png`, fullPage: false });
  console.log('Captured members-list.png');

  // Categories
  await page.goto(`${BASE_URL}/#/archive-categories`, { waitUntil: 'networkidle', timeout: 10000 });
  await sleep(1000);
  await page.screenshot({ path: `${OUTPUT_DIR}/categories-list.png`, fullPage: false });
  console.log('Captured categories-list.png');

  // Tags
  await page.goto(`${BASE_URL}/#/archive-tags`, { waitUntil: 'networkidle', timeout: 10000 });
  await sleep(1000);
  await page.screenshot({ path: `${OUTPUT_DIR}/tags-list.png`, fullPage: false });
  console.log('Captured tags-list.png');

  // Archives
  await page.goto(`${BASE_URL}/#/archives`, { waitUntil: 'networkidle', timeout: 10000 });
  await sleep(1000);
  await page.screenshot({ path: `${OUTPUT_DIR}/archives-stats.png`, fullPage: false });
  console.log('Captured archives-stats.png');

  // Archives list tab
  await page.click('button:has-text("档案列表")');
  await sleep(800);
  await page.screenshot({ path: `${OUTPUT_DIR}/archives-list.png`, fullPage: false });
  console.log('Captured archives-list.png');

  // Borrow form modal
  await page.click('button:has-text("借出")');
  await sleep(800);
  await page.screenshot({ path: `${OUTPUT_DIR}/borrow-form.png`, fullPage: false });
  console.log('Captured borrow-form.png');
  // Close modal by clicking the cancel button
  await page.click('button:has-text("取消")');
  await sleep(500);

  // Borrows list tab
  await page.click('button:has-text("借还记录")');
  await sleep(800);
  await page.screenshot({ path: `${OUTPUT_DIR}/borrows-list.png`, fullPage: false });
  console.log('Captured borrows-list.png');

  // Settings
  await page.goto(`${BASE_URL}/#/settings`, { waitUntil: 'networkidle', timeout: 10000 });
  await sleep(1000);
  await page.screenshot({ path: `${OUTPUT_DIR}/settings-export.png`, fullPage: false });
  console.log('Captured settings-export.png');

  // Mobile server section
  await page.evaluate(() => {
    const headings = Array.from(document.querySelectorAll('h3'));
    const target = headings.find((h) => h.textContent.includes('手机搜索服务'));
    if (target) target.scrollIntoView({ block: 'start' });
  });
  await sleep(500);
  await page.screenshot({ path: `${OUTPUT_DIR}/mobile-server.png`, fullPage: false });
  console.log('Captured mobile-server.png');

  await browser.close();
}

takeScreenshots().catch(console.error);
