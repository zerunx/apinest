package logger

import (
	"os"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

var Logger = zap.NewNop()

func Init() {
	var cores []zapcore.Core

	if os.Getenv("LOG_TO_CONSOLE") != "false" {
		cores = append(cores, newConsoleCore())
	}

	if os.Getenv("LOG_TO_FILE") != "false" {
		cores = append(cores, newInfoCore())
		cores = append(cores, newErrorCore())
	}

	if len(cores) == 0 {
		cores = append(cores, newConsoleCore())
	}

	core := zapcore.NewTee(cores...)

	options := []zap.Option{
		zap.AddCaller(),
		zap.AddCallerSkip(2),
	}

	if os.Getenv("LOG_STACKTRACE") == "true" {
		options = append(options, zap.AddStacktrace(zapcore.ErrorLevel))
	}

	Logger = zap.New(core, options...)
}

func Debug(msg string, args ...any) {
	log(zap.DebugLevel, msg, args...)
}

func Info(msg string, args ...any) {
	log(zap.InfoLevel, msg, args...)
}

func Warn(msg string, args ...any) {
	log(zap.WarnLevel, msg, args...)
}

func Error(msg string, args ...any) {
	log(zap.ErrorLevel, msg, args...)
}

func log(level zapcore.Level, msg string, args ...any) {
	fields := make([]zap.Field, 0, len(args)/2+1)

	var err error
	var kv []any

	if len(args) > 0 {
		if e, ok := args[0].(error); ok {
			err = e
			kv = args[1:]
		} else {
			kv = args
		}
	}

	if err != nil {
		fields = append(fields, zap.Error(err))
	}

	fields = append(fields, parseKV(kv...)...)

	switch level {
	case zap.DebugLevel:
		Logger.Debug(msg, fields...)

	case zap.InfoLevel:
		Logger.Info(msg, fields...)

	case zap.WarnLevel:
		Logger.Warn(msg, fields...)

	case zap.ErrorLevel:
		Logger.Error(msg, fields...)
	}
}

func parseKV(kv ...any) []zap.Field {
	fields := make([]zap.Field, 0, len(kv)/2)

	length := len(kv)

	for i := 0; i < length; i += 2 {

		if i+1 >= length {
			break
		}

		key, ok := kv[i].(string)
		if !ok {
			continue
		}

		fields = append(fields, zap.Any(key, kv[i+1]))
	}

	return fields
}
