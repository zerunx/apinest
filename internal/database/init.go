package database

import (
	"context"
	"database/sql"
	"log"

	"entgo.io/ent/dialect"
	entsql "entgo.io/ent/dialect/sql"

	_ "github.com/jackc/pgx/v5/stdlib"

	"nest-api/configs"
	"nest-api/internal/runtime"
)

func Init() {
	if !runtime.IsInstalled() {
		return
	}

	if err := InitWithConfig(configs.Database); err != nil {
		log.Fatal(err)
	}
}

func InitWithConfig(cfg configs.DatabaseConfig) error {
	db, err := sql.Open("pgx", BuildDSN(cfg))
	if err != nil {
		return err
	}

	if err := db.PingContext(context.Background()); err != nil {
		db.Close()
		return err
	}

	driver := entsql.OpenDB(dialect.Postgres, db)

	initPool(db)

	client := newClient(driver)

	if err := runMigrate(client, db); err != nil {
		db.Close()
		return err
	}

	DB = client
	return nil
}
