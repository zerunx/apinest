package database

import (
	"database/sql"
	"time"

	"nest-api/configs"
)

func initPool(db *sql.DB) {

	cfg := configs.Database

	db.SetMaxIdleConns(cfg.MaxIdleConns)

	db.SetMaxOpenConns(cfg.MaxOpenConns)

	db.SetConnMaxLifetime(
		time.Duration(cfg.ConnMaxLifetime) * time.Second,
	)
}
