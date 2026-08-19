package bootstrap

import (
	"nest-api/internal/database"
)

func InitDatabase() {
	database.Init()
}
