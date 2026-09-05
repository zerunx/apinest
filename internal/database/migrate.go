package database

import (
	"context"
	"database/sql"

	"nest-api/internal/ent"
	"nest-api/internal/ent/migrate"
)

func runMigrate(client *ent.Client, db *sql.DB) error {
	ctx := context.Background()
	if err := client.Schema.Create(
		ctx,
		migrate.WithForeignKeys(false),
		// Drop indexes removed from schema (e.g. full unique on users.account
		// replaced by a partial unique index WHERE deleted_at IS NULL).
		migrate.WithDropIndex(true),
	); err != nil {
		return err
	}
	// Column-level UNIQUE may remain as a table constraint; ensure it is gone
	// so soft-deleted accounts can be reused under the partial index.
	_, err := db.ExecContext(ctx, `ALTER TABLE users DROP CONSTRAINT IF EXISTS users_account_key`)
	return err
}
