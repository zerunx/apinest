package utils

import (
	"os"
	"strconv"
)

// 取环境变量工具函数
func String(key string, fallback string) string {
	val := os.Getenv(key)

	// fmt.Println("CONFIG: ", key, val)
	if val == "" {
		return fallback
	}

	return val
}

func Int(key string, fallback int) int {
	val := os.Getenv(key)

	if val == "" {
		return fallback
	}

	i, err := strconv.Atoi(val)
	if err != nil {
		return fallback
	}

	return i
}
