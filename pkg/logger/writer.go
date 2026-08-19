package logger

import (
	"os"
	"path/filepath"
	"time"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

func newConsoleCore() zapcore.Core {
	level := zap.LevelEnablerFunc(func(l zapcore.Level) bool {
		return true
	})

	return zapcore.NewCore(
		newEncoder(),
		zapcore.AddSync(os.Stdout),
		level,
	)
}

func newInfoCore() zapcore.Core {
	level := zap.LevelEnablerFunc(func(l zapcore.Level) bool {
		return l < zapcore.ErrorLevel
	})

	return zapcore.NewCore(
		newEncoder(),
		newWriter("info"),
		level,
	)
}

func newErrorCore() zapcore.Core {
	level := zap.LevelEnablerFunc(func(l zapcore.Level) bool {
		return l >= zapcore.ErrorLevel
	})

	return zapcore.NewCore(
		newEncoder(),
		newWriter("error"),
		level,
	)
}

func newWriter(name string) zapcore.WriteSyncer {
	date := time.Now().Format("20060102")

	dir := filepath.Join(
		"runtime",
		"logs",
		date,
	)

	_ = os.MkdirAll(dir, os.ModePerm)

	filename := filepath.Join(
		dir,
		name+".log",
	)

	file, err := os.OpenFile(
		filename,
		os.O_CREATE|os.O_APPEND|os.O_WRONLY,
		0666,
	)

	if err != nil {
		panic(err)
	}

	return zapcore.Lock(zapcore.AddSync(file))
}
