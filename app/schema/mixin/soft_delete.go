package mixin

import (
	"context"
	"fmt"

	gen "nest-api/internal/ent"
	"nest-api/internal/ent/hook"
	"nest-api/internal/ent/intercept"
	"nest-api/internal/utils"

	"entgo.io/ent"
	"entgo.io/ent/dialect/sql"
	"entgo.io/ent/schema/mixin"
)

type SoftDeleteMixin struct {
	mixin.Schema
}

type softDeleteKey struct{}

func SkipSoftDelete(parent context.Context) context.Context {
	return context.WithValue(parent, softDeleteKey{}, true)
}

func (SoftDeleteMixin) Interceptors() []ent.Interceptor {
	return []ent.Interceptor{
		intercept.TraverseFunc(func(ctx context.Context, q intercept.Query) error {
			if skip, _ := ctx.Value(softDeleteKey{}).(bool); skip {
				return nil
			}
			filterNotDeleted(q)
			return nil
		}),
	}
}

func (SoftDeleteMixin) Hooks() []ent.Hook {
	return []ent.Hook{
		hook.On(
			func(next ent.Mutator) ent.Mutator {
				return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
					if skip, _ := ctx.Value(softDeleteKey{}).(bool); skip {
						return next.Mutate(ctx, m)
					}
					mx, ok := m.(interface {
						SetOp(ent.Op)
						Client() *gen.Client
						SetDeletedAt(utils.DateTime)
						WhereP(...func(*sql.Selector))
						Op() ent.Op
						ID() (id int64, exists bool)
					})
					if !ok {
						return nil, fmt.Errorf("unexpected mutation type %T", m)
					}
					filterNotDeleted(mx)

					op := ent.OpUpdate
					if mx.Op().Is(ent.OpDeleteOne) {
						if _, exists := mx.ID(); exists {
							op = ent.OpUpdateOne
						}
					}
					mx.SetOp(op)
					mx.SetDeletedAt(utils.Now())
					return mx.Client().Mutate(ctx, m)
				})
			},
			ent.OpDeleteOne|ent.OpDelete,
		),
	}
}

func filterNotDeleted(w interface{ WhereP(...func(*sql.Selector)) }) {
	w.WhereP(sql.FieldIsNull("deleted_at"))
}
