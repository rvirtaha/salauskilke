package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

type PingRequest struct {
	Message string `json:"message" binding:"omitempty"`
}

type PingResponse struct {
	Message string `json:"message"`
}

func CreatePingHandler () gin.HandlerFunc {
	return func(ctx *gin.Context) {
		var req PingRequest
		if err := ctx.ShouldBindJSON(&req); err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}

		ctx.JSON(http.StatusOK, &PingResponse{ Message: req.Message })
	}
}