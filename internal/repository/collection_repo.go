package repository

import (
	"database/sql"
	"time"

	"api-dev/internal/db"
	"api-dev/internal/models"
)

// ListCollections 查询全部集合（可按父级过滤）
func ListCollections(parentID *int64) ([]models.Collection, error) {
	rows, err := db.DB.Query(`SELECT id, parent_id, name, created_at FROM collections ORDER BY name ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var list []models.Collection
	for rows.Next() {
		var c models.Collection
		var parent sql.NullInt64
		var ts string
		if err := rows.Scan(&c.ID, &parent, &c.Name, &ts); err != nil {
			return nil, err
		}
		if parent.Valid {
			pid := parent.Int64
			c.ParentID = &pid
		}
		c.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", ts)
		list = append(list, c)
	}
	return list, nil
}

// CreateCollection 新建集合
func CreateCollection(name string, parentID *int64) (models.Collection, error) {
	res, err := db.DB.Exec(`INSERT INTO collections (parent_id, name) VALUES (?, ?)`, parentID, name)
	if err != nil {
		return models.Collection{}, err
	}
	id, _ := res.LastInsertId()
	return models.Collection{ID: id, ParentID: parentID, Name: name, CreatedAt: time.Now()}, nil
}

// UpdateCollection 重命名集合
func UpdateCollection(id int64, name string) error {
	_, err := db.DB.Exec(`UPDATE collections SET name = ? WHERE id = ?`, name, id)
	return err
}

// DeleteCollection 删除集合（级联删除子集合与下挂请求由数据库外键处理子集合；
// 下挂请求的 collection_id 会被置空，请求本身保留）
func DeleteCollection(id int64) error {
	tx, err := db.DB.Begin()
	if err != nil {
		return err
	}
	// 先把子集合上移到被删集合的父级，避免连带删除
	if _, err := tx.Exec(`UPDATE collections SET parent_id = (SELECT parent_id FROM collections WHERE id = ?) WHERE parent_id = ?`, id, id); err != nil {
		tx.Rollback()
		return err
	}
	if _, err := tx.Exec(`DELETE FROM collections WHERE id = ?`, id); err != nil {
		tx.Rollback()
		return err
	}
	return tx.Commit()
}
