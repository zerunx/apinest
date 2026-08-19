package database

import (
	"context"

	"nest-api/internal/ent"
	"nest-api/internal/ent/migrate"
)

func runMigrate(client *ent.Client) error {
	return client.Schema.Create(
		context.Background(),
		migrate.WithForeignKeys(false),
	)
}
