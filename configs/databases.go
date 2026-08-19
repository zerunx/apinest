package configs

import (
	"nest-api/internal/utils"
)

type DatabaseConfig struct {
	Host            string
	Port            int
	User            string
	Password        string
	DBName          string
	SSLMode         string
	MaxIdleConns    int
	MaxOpenConns    int
	ConnMaxLifetime int
}

var Database = DatabaseConfig{

	Host:     utils.String("DB_HOST", "localhost"),
	Port:     utils.Int("DB_PORT", 5432),
	User:     utils.String("DB_USER", "postgres"),
	Password: utils.String("DB_PASSWORD", "postgres"),
	DBName:   utils.String("DB_NAME", "go_starter"),

	SSLMode:         utils.String("DB_SSLMODE", "disable"),
	MaxIdleConns:    utils.Int("DB_MAX_IDLE_CONNS", 10),
	MaxOpenConns:    utils.Int("DB_MAX_OPEN_CONNS", 100),
	ConnMaxLifetime: utils.Int("DB_CONN_MAX_LIFETIME", 3600),
}
