pub const MIGRATIONS: &[(&str, &str)] = &[
    ("init", r#"
    CREATE TABLE IF NOT EXISTS members (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        phone TEXT,
        email TEXT,
        note TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS tasks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        description TEXT,
        cycle_type TEXT CHECK(cycle_type IN ('monthly','quarterly','halfyearly','yearly')) NOT NULL,
        cycle_day INTEGER NOT NULL,
        start_date DATE NOT NULL,
        end_date DATE,
        reminder_minutes INTEGER DEFAULT 15,
        sound_enabled INTEGER DEFAULT 1,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS task_assignees (
        task_id INTEGER REFERENCES tasks(id) ON DELETE CASCADE,
        member_id INTEGER REFERENCES members(id) ON DELETE CASCADE,
        PRIMARY KEY (task_id, member_id)
    );

    CREATE TABLE IF NOT EXISTS task_instances (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        task_id INTEGER REFERENCES tasks(id) ON DELETE CASCADE,
        due_date DATE NOT NULL,
        status TEXT CHECK(status IN ('pending','completed','overdue')) DEFAULT 'pending',
        confirmed_at DATETIME,
        reminded INTEGER DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE INDEX IF NOT EXISTS idx_instances_due_date ON task_instances(due_date);
    CREATE INDEX IF NOT EXISTS idx_instances_status ON task_instances(status);
    CREATE INDEX IF NOT EXISTS idx_instances_task_id ON task_instances(task_id);
    "#),
    ("archive_base", r#"
    CREATE TABLE IF NOT EXISTS archive_categories (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        code_prefix TEXT NOT NULL,
        note TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS archives (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        code TEXT NOT NULL UNIQUE,
        title TEXT NOT NULL,
        category_id INTEGER REFERENCES archive_categories(id),
        location TEXT,
        keeper_id INTEGER REFERENCES members(id),
        status TEXT CHECK(status IN ('in_stock','borrowed','damaged','destroyed')) DEFAULT 'in_stock',
        quantity INTEGER DEFAULT 1,
        description TEXT,
        photos TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS archive_borrows (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        archive_id INTEGER REFERENCES archives(id) ON DELETE CASCADE,
        borrower_id INTEGER REFERENCES members(id) NOT NULL,
        purpose TEXT,
        borrow_date DATE NOT NULL,
        due_date DATE NOT NULL,
        return_date DATE,
        status TEXT CHECK(status IN ('borrowed','returned','overdue')) DEFAULT 'borrowed',
        approver_id INTEGER REFERENCES members(id),
        note TEXT,
        reminded INTEGER DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE INDEX IF NOT EXISTS idx_archives_code ON archives(code);
    CREATE INDEX IF NOT EXISTS idx_archives_category ON archives(category_id);
    CREATE INDEX IF NOT EXISTS idx_archives_status ON archives(status);
    CREATE INDEX IF NOT EXISTS idx_archive_borrows_archive ON archive_borrows(archive_id);
    CREATE INDEX IF NOT EXISTS idx_archive_borrows_status ON archive_borrows(status);
    CREATE INDEX IF NOT EXISTS idx_archive_borrows_due ON archive_borrows(due_date);

    INSERT OR IGNORE INTO archive_categories (id, name, code_prefix) VALUES
        (1, '业主档案', 'YZ'),
        (2, '设备档案', 'SB'),
        (3, '合同档案', 'HT'),
        (4, '财务档案', 'CW'),
        (5, '人事档案', 'RS');
    "#),
    ("archive_tags", r#"
    CREATE TABLE IF NOT EXISTS archive_tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        parent_id INTEGER REFERENCES archive_tags(id) ON DELETE CASCADE,
        note TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS archive_tag_relations (
        archive_id INTEGER NOT NULL REFERENCES archives(id) ON DELETE CASCADE,
        tag_id INTEGER NOT NULL REFERENCES archive_tags(id) ON DELETE CASCADE,
        PRIMARY KEY (archive_id, tag_id)
    );

    CREATE INDEX IF NOT EXISTS idx_archive_tags_parent ON archive_tags(parent_id);
    CREATE INDEX IF NOT EXISTS idx_archive_tag_relations_archive ON archive_tag_relations(archive_id);
    CREATE INDEX IF NOT EXISTS idx_archive_tag_relations_tag ON archive_tag_relations(tag_id);
    "#),
    ("unknown_defaults", r#"
    ALTER TABLE members ADD COLUMN is_system INTEGER DEFAULT 0;
    ALTER TABLE archive_categories ADD COLUMN is_system INTEGER DEFAULT 0;

    INSERT OR IGNORE INTO archive_categories (id, name, code_prefix, note, is_system)
    VALUES (999999, '未知分类', 'UNKNOWN', '系统默认分类，导入时未指定分类的档案会归到这里', 1);

    INSERT OR IGNORE INTO members (id, name, phone, email, note, is_system)
    VALUES (999999, '未知保管人', NULL, NULL, '系统默认人员，导入时未指定保管人的档案会归到这里', 1);
    "#),
    ("archive_electronic", r#"
    ALTER TABLE archives ADD COLUMN archive_type TEXT DEFAULT 'paper';
    ALTER TABLE archives ADD COLUMN box_name TEXT;
    ALTER TABLE archives ADD COLUMN file_path TEXT;

    UPDATE archives SET archive_type = 'paper' WHERE archive_type IS NULL;
    "#),
];
