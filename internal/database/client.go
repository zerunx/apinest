package database

import (
	"nest-api/internal/ent"

	entsql "entgo.io/ent/dialect/sql"
)

var DB *ent.Client

func newClient(driver *entsql.Driver) *ent.Client {
	return ent.NewClient(
		ent.Driver(driver),
	)
}
