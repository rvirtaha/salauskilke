package router

import (
	"salauskilke/internal/generated/db"

	"github.com/gin-gonic/gin"

	"salauskilke/internal/handlers"
)

func SetupRouter(q *db.Queries) *gin.Engine {
	engine := gin.Default()
	engine.SetTrustedProxies(nil)

	api := engine.Group("/api")
	{
		api.POST("/ping", handlers.CreatePingHandler())
	}

	return engine
}
