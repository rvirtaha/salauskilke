package router

import (
	"salauskilke/internal/generated/db"

	"github.com/gin-gonic/gin"

	"salauskilke/internal/handlers"
)

func SetupRouter(q *db.Queries) *gin.Engine {
	engine := gin.Default()
	engine.SetTrustedProxies(nil)

	engine.LoadHTMLGlob("internal/templates/**/*")

	engine.Static("/static", "internal/static")

	api := engine.Group("/api")
	{
		api.POST("/ping", handlers.CreatePingHandler())
	}

	views := engine.Group("/")
	{
		views.GET("/", handlers.RootHandler)
		views.GET("/hello", handlers.HelloHandler)
	}

	return engine
}
