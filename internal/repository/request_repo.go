package repository

import (
	"database/sql"
	"encoding/json"
	"time"

	"api-dev/internal/db"
	"api-dev/internal/models"
)

// ListRequests 查询请求列表（可按集合过滤）
func ListRequests(collectionID *int64) ([]models.Request, error) {
	var rows *sql.Rows
	var err error
	if collectionID == nil {
		rows, err = db.DB.Query(`SELECT id, collection_id, name, method, url, headers, params, body_type, body, tests_script, created_at FROM requests ORDER BY name ASC`)
	} else {
		rows, err = db.DB.Query(`SELECT id, collection_id, name, method, url, headers, params, body_type, body, tests_script, created_at FROM requests WHERE collection_id = ? ORDER BY name ASC`, *collectionID)
	}
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var list []models.Request
	for rows.Next() {
		r, err := scanRequest(rows)
		if err != nil {
			return nil, err
		}
		list = append(list, r)
	}
	return list, nil
}

// GetRequest 按 ID 查询单个请求
func GetRequest(id int64) (models.Request, error) {
	row := db.DB.QueryRow(`SELECT id, collection_id, name, method, url, headers, params, body_type, body, tests_script, created_at FROM requests WHERE id = ?`, id)
	return scanRequestRow(row)
}

// SaveRequest 新建或更新请求（id>0 视为更新）
func SaveRequest(r models.Request) (models.Request, error) {
	headers, _ := json.Marshal(emptyIfNil(r.Headers))
	params, _ := json.Marshal(emptyIfNil(r.Params))

	if r.ID > 0 {
		_, err := db.DB.Exec(`UPDATE requests SET collection_id=?, name=?, method=?, url=?, headers=?, params=?, body_type=?, body=?, tests_script=? WHERE id=?`,
			r.CollectionID, r.Name, r.Method, r.URL, string(headers), string(params), r.BodyType, r.Body, r.TestsScript, r.ID)
		if err != nil {
			return r, err
		}
		return r, nil
	}

	res, err := db.DB.Exec(`INSERT INTO requests (collection_id, name, method, url, headers, params, body_type, body, tests_script) VALUES (?,?,?,?,?,?,?,?,?)`,
		r.CollectionID, r.Name, r.Method, r.URL, string(headers), string(params), r.BodyType, r.Body, r.TestsScript)
	if err != nil {
		return r, err
	}
	r.ID, _ = res.LastInsertId()
	r.CreatedAt = time.Now()
	return r, nil
}

// DeleteRequest 删除请求
func DeleteRequest(id int64) error {
	_, err := db.DB.Exec(`DELETE FROM requests WHERE id = ?`, id)
	return err
}

// scanRequest 通用行扫描
func scanRequest(rows *sql.Rows) (models.Request, error) {
	var r models.Request
	var cid sql.NullInt64
	var headers, params, ts string
	if err := rows.Scan(&r.ID, &cid, &r.Name, &r.Method, &r.URL, &headers, &params, &r.BodyType, &r.Body, &r.TestsScript, &ts); err != nil {
		return r, err
	}
	if cid.Valid {
		pid := cid.Int64
		r.CollectionID = &pid
	}
	_ = json.Unmarshal([]byte(headers), &r.Headers)
	_ = json.Unmarshal([]byte(params), &r.Params)
	r.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", ts)
	return r, nil
}

func scanRequestRow(row *sql.Row) (models.Request, error) {
	var r models.Request
	var cid sql.NullInt64
	var headers, params, ts string
	if err := row.Scan(&r.ID, &cid, &r.Name, &r.Method, &r.URL, &headers, &params, &r.BodyType, &r.Body, &r.TestsScript, &ts); err != nil {
		return r, err
	}
	if cid.Valid {
		pid := cid.Int64
		r.CollectionID = &pid
	}
	_ = json.Unmarshal([]byte(headers), &r.Headers)
	_ = json.Unmarshal([]byte(params), &r.Params)
	r.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", ts)
	return r, nil
}

func emptyIfNil(kvs []models.KV) []models.KV {
	if kvs == nil {
		return []models.KV{}
	}
	return kvs
}
