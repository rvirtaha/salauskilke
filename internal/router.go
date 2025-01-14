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
		api.POST("/register/initialize", handlers.CreateInitializeRegistrationHandler(q, opaqueServer, opaqueSetup))
		api.POST("/register/finalize", handlers.CreateFinalizeRegistrationHandler(q, opaqueServer))
	}

	return engine
}
