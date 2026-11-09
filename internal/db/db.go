package db

import (
	"database/sql"
	"os"
	"path/filepath"

	_ "modernc.org/sqlite"
)

// DB 全局数据库连接
var DB *sql.DB

// Init 初始化 SQLite 数据库（数据文件存于运行目录下的 db/data.db）
func Init() error {
	dbDir := "db"
	if err := os.MkdirAll(dbDir, 0755); err != nil {
		return err
	}
	dbPath := filepath.Join(dbDir, "data.db")

	d, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return err
	}
	// SQLite 启用外键约束
	if _, err := d.Exec("PRAGMA foreign_keys = ON;"); err != nil {
		return err
	}
	DB = d
	return applySchema()
}

func applySchema() error {
	_, err := DB.Exec(schema)
	return err
}

const schema = `
CREATE TABLE IF NOT EXISTS collections (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	parent_id INTEGER REFERENCES collections(id) ON DELETE CASCADE,
	name TEXT NOT NULL,
	created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS requests (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL,
	name TEXT NOT NULL DEFAULT '',
	method TEXT NOT NULL DEFAULT 'GET',
	url TEXT NOT NULL DEFAULT '',
	headers TEXT NOT NULL DEFAULT '[]',
	params TEXT NOT NULL DEFAULT '[]',
	body_type TEXT NOT NULL DEFAULT 'none',
	body TEXT NOT NULL DEFAULT '',
	tests_script TEXT NOT NULL DEFAULT '',
	created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS environments (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL,
	is_active INTEGER NOT NULL DEFAULT 0,
	created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS variables (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	environment_id INTEGER NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
	key TEXT NOT NULL,
	value TEXT NOT NULL DEFAULT '',
	type TEXT NOT NULL DEFAULT 'string'
);
`
