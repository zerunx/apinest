package interfaces

import (
	"context"

	"nest-api/internal/database"
	entapi "nest-api/internal/ent/api"
)

// BackfillLegacyStatuses maps removed statuses (4 异常, 5 维护) to 开发中(3).
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
