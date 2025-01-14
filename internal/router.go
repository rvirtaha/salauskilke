package router

import (
	"salauskilke/internal/generated/db"
	"salauskilke/internal/utils"

	"github.com/bytemare/opaque"
	"github.com/gin-gonic/gin"

	"salauskilke/internal/handlers"
)

func SetupRouter(
		q *db.Queries, 
		opaqueServer *opaque.Server, 
		opaqueSetup *utils.OpaqueSetupType,
	) *gin.Engine {
	
	engine := gin.Default()
	engine.SetTrustedProxies(nil)

	api := engine.Group("/api")
	{
		api.POST("/ping", handlers.CreatePingHandler())
	}

	return engine
}
