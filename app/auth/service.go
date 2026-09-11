package auth

import (
	"context"
	"regexp"
	"strings"

	"nest-api/internal/auth"
	"nest-api/internal/database"
	"nest-api/internal/ent"
	"nest-api/internal/ent/user"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

var accountPattern = regexp.MustCompile(`^[a-zA-Z0-9_]+$`)

type Service struct{}

func (Service) Register(ctx context.Context, params RegisterRequest) (LoginResponse, error) {
	if params.Password != params.ConfirmPassword {
		return LoginResponse{}, bizerr.New("passwords do not match")
	}

	account := strings.TrimSpace(params.Account)
	if !accountPattern.MatchString(account) {
		return LoginResponse{}, bizerr.New("account may only contain letters, digits, and underscores")
	}

	email := strings.TrimSpace(strings.ToLower(params.Email))

	exists, err := database.DB.User.
		Query().
		Where(user.AccountEQ(account)).
		Exist(ctx)
	if err != nil {
		return LoginResponse{}, err
	}
	if exists {
		return LoginResponse{}, bizerr.New("account already exists")
	}

	if email != "" {
		exists, err = database.DB.User.
			Query().
			Where(user.EmailEQ(email)).
			Exist(ctx)
		if err != nil {
			return LoginResponse{}, err
		}
		if exists {
			return LoginResponse{}, bizerr.New("email is already in use")
		}
	}

	hash, err := utils.Hash(params.Password)
	if err != nil {
		return LoginResponse{}, err
	}

	u, err := database.DB.User.
		Create().
		SetName(strings.TrimSpace(params.Name)).
		SetAccount(account).
		SetEmail(email).
		SetPassword(hash).
		SetStatus(1).
		Save(ctx)
	if err != nil {
		return LoginResponse{}, err
	}

	return issueTokenPair(u)
}

func (Service) Login(ctx context.Context, params LoginRequest) (LoginResponse, error) {
	account := strings.TrimSpace(params.Account)

	u, err := database.DB.User.
		Query().
		Where(user.AccountEQ(account)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return LoginResponse{}, bizerr.New("invalid account or password")
		}
		return LoginResponse{}, err
	}

	if u.Status != 1 {
		return LoginResponse{}, bizerr.New("account is disabled")
	}

	if !utils.Verify(params.Password, u.Password) {
		return LoginResponse{}, bizerr.New("invalid account or password")
	}

	return issueTokenPair(u)
}

func (Service) Me(ctx context.Context, userID int64) (UserProfile, error) {
	u, err := database.DB.User.
		Query().
		Where(user.IDEQ(userID)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return UserProfile{}, bizerr.New("user not found")
		}
		return UserProfile{}, err
	}

	return toProfile(u), nil
}

func issueTokenPair(u *ent.User) (LoginResponse, error) {
	access, refresh, err := auth.JWT.GenerateTokenPair(u.ID)
	if err != nil {
		return LoginResponse{}, err
	}

	return LoginResponse{
		AccessToken:  access,
		RefreshToken: refresh,
		User:         toProfile(u),
	}, nil
}

func toProfile(u *ent.User) UserProfile {
	return UserProfile{
		ID:      u.ID,
		Name:    u.Name,
		Account: u.Account,
		Email:   u.Email,
		Avatar:  u.Avatar,
		IsAdmin: u.IsAdmin,
		Status:  u.Status,
	}
}
