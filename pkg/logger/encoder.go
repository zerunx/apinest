package logger

import (
	"time"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

func newEncoder() zapcore.Encoder {
	cfg := zap.NewProductionEncoderConfig()

	cfg.TimeKey = "time"
	cfg.LevelKey = "level"
	cfg.MessageKey = "msg"
	cfg.CallerKey = "caller"

	cfg.EncodeTime = func(t time.Time, enc zapcore.PrimitiveArrayEncoder) {
		enc.AppendString(
			t.Format("2006-01-02 15:04:05"),
		)
	}

	cfg.EncodeLevel = zapcore.CapitalLevelEncoder
	cfg.EncodeCaller = zapcore.ShortCallerEncoder

	return zapcore.NewConsoleEncoder(cfg)
	// return zapcore.NewJSONEncoder(cfg)
}
