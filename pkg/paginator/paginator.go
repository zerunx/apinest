package paginator

import "context"

type Meta struct {
	Total int `json:"total"`
	Page  int `json:"page"`
	Size  int `json:"size"`
}

type Query[Q any, T any] interface {
	Count(ctx context.Context) (int, error)
	Offset(int) Q
	Limit(int) Q
	All(ctx context.Context) ([]T, error)
}

func Paginate[Q Query[Q, T], T any](ctx context.Context, query Q, page, size int) ([]T, *Meta, error) {
	if page <= 0 {
		page = 1
	}
	if size <= 0 {
		size = 10
	}

	total, err := query.Count(ctx)
	if err != nil {
		return nil, nil, err
	}

	offset := (page - 1) * size
	list, err := query.Offset(offset).Limit(size).All(ctx)
	if err != nil {
		return nil, nil, err
	}

	return list, &Meta{
		Total: total,
		Page:  page,
		Size:  size,
	}, nil
}
