package main

import (
	bootstarp "nest-api/bootstarp"
	_ "nest-api/internal/ent/runtime"
)

func main() {
	bootstarp.Boot()
}
