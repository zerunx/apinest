package interfaces

import (
	"context"

	"nest-api/internal/database"
	entapi "nest-api/internal/ent/api"
)

func BackfillLegacyStatuses(ctx context.Context) error {
	if database.DB == nil {
		return nil
	}
	_, err := database.DB.API.
		Update().
		Where(entapi.StatusIn(4, 5)).
		SetStatus(3).
		Save(ctx)
	return err
}
