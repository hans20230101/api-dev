package repository

import (
	"database/sql"
	"time"

	"api-dev/internal/db"
	"api-dev/internal/models"
)

// ListEnvironments 查询全部环境
func ListEnvironments() ([]models.Environment, error) {
	rows, err := db.DB.Query(`SELECT id, name, is_active, created_at FROM environments ORDER BY name ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var list []models.Environment
	for rows.Next() {
		var e models.Environment
		var active int
		var ts string
		if err := rows.Scan(&e.ID, &e.Name, &active, &ts); err != nil {
			return nil, err
		}
		e.IsActive = active == 1
		e.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", ts)
		list = append(list, e)
	}
	return list, nil
}

// CreateEnvironment 新建环境
func CreateEnvironment(name string) (models.Environment, error) {
	res, err := db.DB.Exec(`INSERT INTO environments (name) VALUES (?)`, name)
	if err != nil {
		return models.Environment{}, err
	}
	id, _ := res.LastInsertId()
	return models.Environment{ID: id, Name: name, CreatedAt: time.Now()}, nil
}

// UpdateEnvironment 重命名环境
func UpdateEnvironment(id int64, name string) error {
	_, err := db.DB.Exec(`UPDATE environments SET name = ? WHERE id = ?`, name, id)
	return err
}

// DeleteEnvironment 删除环境（变量由外键级联删除）
func DeleteEnvironment(id int64) error {
	_, err := db.DB.Exec(`DELETE FROM environments WHERE id = ?`, id)
	return err
}

// SetActiveEnvironment 设置当前激活环境（同时取消其它激活）
func SetActiveEnvironment(id int64) error {
	tx, err := db.DB.Begin()
	if err != nil {
		return err
	}
	if _, err := tx.Exec(`UPDATE environments SET is_active = 0`); err != nil {
		tx.Rollback()
		return err
	}
	if _, err := tx.Exec(`UPDATE environments SET is_active = 1 WHERE id = ?`, id); err != nil {
		tx.Rollback()
		return err
	}
	return tx.Commit()
}

// GetActiveEnvironment 获取当前激活环境
func GetActiveEnvironment() (*models.Environment, error) {
	row := db.DB.QueryRow(`SELECT id, name, is_active, created_at FROM environments WHERE is_active = 1 LIMIT 1`)
	var e models.Environment
	var active int
	var ts string
	if err := row.Scan(&e.ID, &e.Name, &active, &ts); err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}
	e.IsActive = active == 1
	e.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", ts)
	return &e, nil
}

// ListVariables 查询某环境的全部变量
func ListVariables(envID int64) ([]models.Variable, error) {
	rows, err := db.DB.Query(`SELECT id, environment_id, key, value, type FROM variables WHERE environment_id = ? ORDER BY key ASC`, envID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var list []models.Variable
	for rows.Next() {
		var v models.Variable
		if err := rows.Scan(&v.ID, &v.EnvironmentID, &v.Key, &v.Value, &v.Type); err != nil {
			return nil, err
		}
		list = append(list, v)
	}
	return list, nil
}

// SaveVariable 新建或更新变量（id>0 视为更新）
func SaveVariable(v models.Variable) (models.Variable, error) {
	if v.ID > 0 {
		_, err := db.DB.Exec(`UPDATE variables SET environment_id=?, key=?, value=?, type=? WHERE id=?`, v.EnvironmentID, v.Key, v.Value, v.Type, v.ID)
		return v, err
	}
	res, err := db.DB.Exec(`INSERT INTO variables (environment_id, key, value, type) VALUES (?,?,?,?)`, v.EnvironmentID, v.Key, v.Value, v.Type)
	if err != nil {
		return v, err
	}
	v.ID, _ = res.LastInsertId()
	return v, nil
}

// DeleteVariable 删除变量
func DeleteVariable(id int64) error {
	_, err := db.DB.Exec(`DELETE FROM variables WHERE id = ?`, id)
	return err
}

// GetActiveVariables 获取当前激活环境的全部变量（无激活环境返回空）
func GetActiveVariables() ([]models.Variable, error) {
	env, err := GetActiveEnvironment()
	if err != nil {
		return nil, err
	}
	if env == nil {
		return []models.Variable{}, nil
	}
	return ListVariables(env.ID)
}
