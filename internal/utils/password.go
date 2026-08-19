package utils

import "golang.org/x/crypto/bcrypt"

func Hash(raw string) (string, error) {

	bytes, err := bcrypt.GenerateFromPassword(
		[]byte(raw),
		bcrypt.DefaultCost,
	)

	if err != nil {
		return "", err
	}

	return string(bytes), nil
}

func Verify(raw string, hash string) bool {

	err := bcrypt.CompareHashAndPassword(
		[]byte(hash),
		[]byte(raw),
	)

	return err == nil
}
