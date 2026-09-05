package bootstrap

import (
	"context"
	"log"

	"nest-api/app/interfaces"
	"nest-api/app/project"
	"nest-api/internal/database"
)

func InitDatabase() {
	database.Init()
	if database.DB == nil {
		return
	}
	if err := project.BackfillMembersForExistingProjects(context.Background()); err != nil {
		log.Printf("backfill project members: %v", err)
	}
	if err := interfaces.BackfillLegacyStatuses(context.Background()); err != nil {
		log.Printf("backfill interface statuses: %v", err)
	}
}
