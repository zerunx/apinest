package user

import (
	"context"

	"nest-api/internal/database"
	"nest-api/internal/ent"
	"nest-api/internal/ent/user"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
	"nest-api/pkg/logger"
	"nest-api/pkg/paginator"
)

type UserService struct{}

func (c UserService) Create(ctx context.Context, params CreateRequest) error {

	exist, err := database.DB.User.
		Query().
		Where(user.MobileEQ(params.Mobile)).
		Exist(ctx)

	if err != nil {
		return err
	}

	if exist {
		return bizerr.New("用户已存在")
	}
	// hash, err := utils.Hash(params.Password)

	_, err = database.DB.User.
		Create().
		SetMobile(params.Mobile).
		SetName(params.Name).
		// SetPassword(hash).
		Save(ctx)

	if err != nil {
		return err
	}

	return nil
}

func (s UserService) List(ctx context.Context, params ListRequest) ([]*ent.User, *paginator.Meta, error) {
	query := database.DB.User.Query()
	if params.Name != "" {
		query = query.Where(user.NameContains(params.Name))
	}
	if params.Mobile != "" {
		query = query.Where(user.MobileContains(params.Mobile))
	}

	return paginator.Paginate(ctx, query, params.Page, params.Size)
}

func (s UserService) Items(ctx context.Context, params ListRequest) ([]*ent.User, error) {
	query := database.DB.User.Query()
	if params.Name != "" {
		query = query.Where(user.NameContains(params.Name))
	}
	if params.Mobile != "" {
		query = query.Where(user.MobileContains(params.Mobile))
	}

	return query.All(ctx)
}

func (c UserService) Update(ctx context.Context, params UpdateRequest) error {

	u, err := database.DB.User.
		Query().
		Where(user.IDEQ(params.ID)).
		Only(ctx)

	if err != nil {
		if ent.IsNotFound(err) {
			return bizerr.New("用户不存在")
		}
		return err
	}

	if params.Mobile != "" && params.Mobile != u.Mobile {

		exist, err := database.DB.User.
			Query().
			Where(
				user.MobileEQ(params.Mobile),
				user.IDNEQ(params.ID),
			).
			Exist(ctx)

		if err != nil {
			return err
		}

		if exist {
			return bizerr.New("手机号已存在")
		}
	}

	_, err = database.DB.User.
		UpdateOneID(params.ID).
		SetName(params.Name).
		SetMobile(params.Mobile).
		Save(ctx)

	if err != nil {
		return err
	}

	return nil
}

func (c UserService) Delete(ctx context.Context, params DeleteRequest) error {

	_, err := database.DB.User.
		UpdateOneID(params.ID).
		SetDeletedAt(utils.Now()).
		Save(ctx)

	if err != nil {
		return err
	}

	return nil
}

func (s UserService) LogActiveUsers(ctx context.Context) error {
	logger.Info("user minute log")
	return nil
}
