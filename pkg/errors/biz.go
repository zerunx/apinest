package errors

import stderrors "errors"

func New(msg string) error {
	return &BizError{
		Code:    CodeFailed,
		Message: msg,
	}
}

func IsBizError(err error) (*BizError, bool) {
	if err == nil {
		return nil, false
	}

	var e *BizError

	ok := stderrors.As(err, &e)

	return e, ok
}
