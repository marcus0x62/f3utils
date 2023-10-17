package debug

import (
	"log"
	"os"
	"strings"
)

type status int

const (
	LogLevelDebug status = 3
	LogLevelWarn  status = 2
	LogLevelInfo  status = 1
	LogLevelOff   status = 0
)

var current_level status = 0

func LogInit() {
	level := os.Getenv("debug")

	switch strings.ToLower(level) {
	case "debug":
		current_level = LogLevelDebug
	case "warn":
		current_level = LogLevelWarn
	case "info":
		current_level = LogLevelInfo
	default:
		log.Println("Log level either not specified or invalid; defaulting to off")
		current_level = LogLevelOff
	}
}

func LogPrint(msg string, level status) {
	if current_level >= level && current_level > 0 {
		log.Println(msg)
	}
}
