package user

type CreateRequest struct {
	Name     string `json:"name" form:"name" binding:"required"`
	Mobile   string `json:"mobile" form:"mobile" binding:"required"`
	Password string `json:"password" form:"password" binding:"required"`
}

type ListRequest struct {
	Name   string
	Mobile string
	Page   int
	Size   int
}

type UpdateRequest struct {
	ID     int64
	Name   string
	Mobile string
}

type DeleteRequest struct {
	ID int64
}

type Info struct {
	ID      int64  `json:"id"`
	Name    string `json:"name"`
	Mobile  string `json:"mobile"`
	IsAdmin bool   `json:"is_admin"`
	Status  int8   `json:"status"`
}
