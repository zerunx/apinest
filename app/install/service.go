package install

import (
	"context"
	"fmt"
	"regexp"
	"strings"
	"sync"
	"time"

	"nest-api/configs"
	"nest-api/internal/database"
	"nest-api/internal/runtime"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

var pgRoleNamePattern = regexp.MustCompile(`^[a-z][a-z0-9_]*$`)
var accountPattern = regexp.MustCompile(`^[a-zA-Z0-9_]+$`)

type Service struct{}

var installMu sync.Mutex

func (Service) Status() StatusResponse {
	return StatusResponse{
		Installed: runtime.IsInstalled(),
		SiteURL:   runtime.SiteURL(),
	}
}

func (s Service) TestDatabase(params TestDatabaseRequest) (TestDatabaseResponse, error) {
	if params.Database.Driver == "mysql" {
		return TestDatabaseResponse{}, bizerr.New("MySQL will be supported in a later version; please use PostgreSQL for now")
	}

	cfg := toDatabaseConfig(params.Database)

	if err := database.TestConnection(cfg); err == nil {
		return TestDatabaseResponse{
			OK:      true,
			Message: "数据库连接成功",
		}, nil
	}

	adminCfg := cfg
	adminCfg.DBName = "postgres"

	if err := database.TestConnection(adminCfg); err != nil {
		return TestDatabaseResponse{}, bizerr.New("cannot connect to the database; check host, port, username, and password")
	}

	return TestDatabaseResponse{
		OK:      true,
		Message: "数据库服务连接成功，目标库不存在时将在安装过程中自动创建",
	}, nil
}

func (s Service) Install(ctx context.Context, params InstallRequest) (InstallResponse, error) {
	installMu.Lock()
	defer installMu.Unlock()

	if runtime.IsInstalled() {
		return InstallResponse{}, bizerr.New("system is already installed")
	}

	if params.Database.Driver == "mysql" {
		return InstallResponse{}, bizerr.New("MySQL will be supported in a later version; please use PostgreSQL for now")
	}

	if params.Admin.Password != params.Admin.ConfirmPassword {
		return InstallResponse{}, bizerr.New("admin passwords do not match")
	}

	adminAccount := strings.TrimSpace(params.Admin.Username)
	if !accountPattern.MatchString(adminAccount) {
		return InstallResponse{}, bizerr.New("admin account may only contain letters, digits, and underscores")
	}

	appUser := params.AppDatabase.Username
	appPassword := params.AppDatabase.Password

	if !pgRoleNamePattern.MatchString(appUser) {
		return InstallResponse{}, bizerr.New("app database username must start with a lowercase letter and contain only lowercase letters, digits, and underscores")
	}

	if appUser == params.Database.User {
		return InstallResponse{}, bizerr.New("app database username cannot be the same as the superuser name")
	}

	cfg := toDatabaseConfig(params.Database)

	if err := database.EnsureDatabase(cfg); err != nil {
		return InstallResponse{}, bizerr.New(fmt.Sprintf("failed to create database: %v", err))
	}

	if err := database.EnsureAppUser(cfg, appUser, appPassword); err != nil {
		return InstallResponse{}, bizerr.New(fmt.Sprintf("failed to create app database user: %v", err))
	}

	appCfg := cfg
	appCfg.User = appUser
	appCfg.Password = appPassword

	if err := database.InitWithConfig(appCfg); err != nil {
		return InstallResponse{}, bizerr.New(fmt.Sprintf("failed to initialize database: %v", err))
	}

	hash, err := utils.Hash(params.Admin.Password)
	if err != nil {
		return InstallResponse{}, err
	}

	_, err = database.DB.User.
		Create().
		SetName(params.Admin.Username).
		SetAccount(adminAccount).
		SetPassword(hash).
		SetIsAdmin(true).
		SetStatus(1).
		Save(ctx)
	if err != nil {
		return InstallResponse{}, bizerr.New(fmt.Sprintf("failed to create admin: %v", err))
	}

	runtimeCfg := configs.RuntimeConfig{
		Installed:   true,
		InstalledAt: time.Now().Format(time.RFC3339),
		SiteURL:     strings.TrimRight(params.SiteURL, "/"),
		Database: configs.DatabaseRuntime{
			Driver:   params.Database.Driver,
			Host:     params.Database.Host,
			Port:     params.Database.Port,
			User:     appUser,
			Password: appPassword,
			Name:     params.Database.Name,
			SSLMode:  params.Database.SSLMode,
		},
	}

	if err := runtime.Save(runtimeCfg); err != nil {
		return InstallResponse{}, bizerr.New(fmt.Sprintf("failed to save config: %v", err))
	}

	return InstallResponse{
		AdminUsername:    params.Admin.Username,
		AdminPassword:    params.Admin.Password,
		DatabaseUser:     appUser,
		DatabasePassword: appPassword,
	}, nil
}

func toDatabaseConfig(db DatabaseRequest) configs.DatabaseConfig {
	return configs.DatabaseConfig{
		Host:     db.Host,
		Port:     db.Port,
		User:     db.User,
		Password: db.Password,
		DBName:   db.Name,
		SSLMode:  db.SSLMode,
	}
}
