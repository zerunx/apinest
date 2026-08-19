package bootstrap

func Boot() {
	InitLogger()
	InitRuntime()
	InitAuth()
	InitValidator()
	InitDatabase()
	InitRedis()
	InitScheduler()

	InitHTTP()
}
