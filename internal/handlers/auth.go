package handlers

import (
	"log"
	"net/http"
	"salauskilke/internal/generated/db"
	"salauskilke/internal/utils"
	"time"

	"github.com/bytemare/opaque"

	"github.com/gin-gonic/gin"
)

type InitializeRegistrationRequest struct {
	RegistrationMessage string `form:"registration_message" json:"registration_message" xml:"registration_message" binding:"required"`
}

type InitializeRegistrationResponse struct {
	ResponseMessage string `json:"response_message"` // unpadded, url-encoded base64 []byte
	CredentialID string `json:"credential_id"` // unpadded, url-encoded base64 []byte
}

func CreateInitializeRegistrationHandler (q *db.Queries, opaqueServer *opaque.Server, opaqueSetup *utils.OpaqueSetupType) gin.HandlerFunc {
	return func(ctx *gin.Context) {
		var req InitializeRegistrationRequest
		if err := ctx.ShouldBindJSON(&req); err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}
		
		// Parse URL-encoded base64 message from client
		registrationRequest, err := utils.DecodeBase64(req.RegistrationMessage)
		if err != nil {
		    ctx.JSON(http.StatusBadRequest, gin.H{"error": err})
		    return
		}

		// Deserialize the client's registration request
		opaqueRegRequest, err := opaqueServer.Deserialize.RegistrationRequest(registrationRequest)
		if err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": "Invalid opaque registration message"})
			return
		}

		// Generate a unique credential identifier for the client
		credID := opaque.RandomBytes(64)
		pks, err := opaqueServer.Deserialize.DecodeAkePublicKey(opaqueSetup.ServerPublicKey)
		if err != nil {
			ctx.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to decode server public key"})
			return
		}

		// Create the server's registration response
		response := opaqueServer.RegistrationResponse(opaqueRegRequest, pks, credID, opaqueSetup.SecretOprfSeed)
		responseMessage := response.Serialize()
		
		// Encode to unpadded, URL-safe Base64
		encodedResponseMessage := utils.EncodeBase64(responseMessage)
		encodedCredID := utils.EncodeBase64(credID)
		
		responseBody := InitializeRegistrationResponse{
			ResponseMessage: encodedResponseMessage,
			CredentialID:    encodedCredID,
		}

		// Send the response back to the client
		ctx.JSON(http.StatusOK, responseBody)

	}
}

type FinalizeRegistrationRequest struct {
	Username       		string `form:"username" json:"username" xml:"username" binding:"required"`
	RegistrationRecord 	string `form:"registration_record" json:"registration_record" xml:"registration_record" binding:"required"`
	CredentialID 		string `form:"credential_id" json:"credential_id" xml:"credential_id" binding:"required"`
}

type FinalizeRegistrationResponse struct {
	Message string `json:"message"`
	UserID int32 `json:"user_id"`
	Username string `json:"username"`
	Created_at string `json:"created_at"`
}

func CreateFinalizeRegistrationHandler (q *db.Queries, opaqueServer *opaque.Server) gin.HandlerFunc {
	return func(ctx *gin.Context) {
		var req FinalizeRegistrationRequest
		if err := ctx.ShouldBindJSON(&req); err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}

		// Parse URL-encoded base64 message from client
		registrationRecord, err := utils.DecodeBase64(req.RegistrationRecord)
		if err != nil {
		    ctx.JSON(http.StatusBadRequest, gin.H{"error": err})
		    return
		}

		// Deserialize the client's registration record
		_, err = opaqueServer.Deserialize.RegistrationRecord(registrationRecord)
		if err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": "Invalid opaque registration record"})
			return
		}
	
		// Save to the database
		userParams := db.InsertUserParams{
			Username: req.Username,
			RegistrationRecord: registrationRecord,
			CredentialIdentifier: []byte(req.CredentialID),
		}

		appUser, err := q.InsertUser(ctx, userParams)
		if err != nil {
			log.Printf("Failed to save user to database: %v", err)
			ctx.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to save registration data"})
			return
		}
		
		responseBody := FinalizeRegistrationResponse{
			Message: "Registration successful",
			UserID: appUser.ID,
			Username: appUser.Username,
			Created_at: appUser.CreatedAt.Time.Format(time.DateTime),
		}

		// Respond with success and user details
		ctx.JSON(http.StatusOK, responseBody)
	}
}
