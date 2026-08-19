package errors

type BizError struct {
	Code    int
	Message string
}

func (e *BizError) Error() string {
	return e.Message
}
