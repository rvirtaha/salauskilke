package server

import (
	"salauskilke/internal/generated/db"

	"github.com/gin-gonic/gin"
)

func SetupRouter(q *db.Queries) *gin.Engine {
	engine := gin.Default()
	engine.SetTrustedProxies(nil)

	api := engine.Group("/api")
	{
		api.POST("/ping", CreatePingHandler())
	}

	return engine
}
