package router

import (
	"salauskilke/internal/generated/db"
	"salauskilke/internal/utils"

	"github.com/gin-gonic/gin"

	"salauskilke/internal/handlers"
)

func SetupRouter(
		q *db.Queries,
		opaqueSetup *utils.OpaqueSetupType,
	) *gin.Engine {
	
	engine := gin.Default()
	engine.SetTrustedProxies(nil)

	api := engine.Group("/api")
	{
		api.POST("/ping", handlers.CreatePingHandler())
		api.POST("/register/initialize", handlers.CreateInitializeRegistrationHandler(q, opaqueSetup.Server, opaqueSetup))
		api.POST("/register/finalize", handlers.CreateFinalizeRegistrationHandler(q, opaqueSetup.Server))
		api.POST("/login/initialize", handlers.CreateInitializeLoginHandler(q, opaqueSetup.Server))
		api.POST("/login/finalize", handlers.CreateFinalizeLoginHandler(opaqueSetup.Server))
		api.GET("/opaqueconf", handlers.CreateGetOpaqueConfHandler(opaqueSetup))
	}

	return engine
}
