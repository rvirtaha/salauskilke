package handlers

import (
	"log"
	"net/http"
	"salauskilke/internal/generated/db"
	"salauskilke/internal/utils"

	"github.com/bytemare/opaque"

	"github.com/gin-gonic/gin"
)

type InitializeRegistrationRequest struct {
	RegistrationMessage string `form:"registration_message" json:"registration_message" xml:"registration_message" binding:"required"`
}

type InitializeRegistrationResponse struct {
	ResponseMessage string `json:"response_message"` // unpadded, url-encoded base64 []byte
	CredentialID    string `json:"credential_id"`    // unpadded, url-encoded base64 []byte
}

func CreateInitializeRegistrationHandler(q *db.Queries, opaqueServer *opaque.Server, opaqueSetup *utils.OpaqueSetupType) gin.HandlerFunc {
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
	Username           string `form:"username" json:"username" xml:"username" binding:"required"`
	RegistrationRecord string `form:"registration_record" json:"registration_record" xml:"registration_record" binding:"required"`
	CredentialID       string `form:"credential_id" json:"credential_id" xml:"credential_id" binding:"required"`
}

type FinalizeRegistrationResponse struct {
	Message  string `json:"message"`
	UserID   int32  `json:"user_id"`
	Username string `json:"username"`
}

func CreateFinalizeRegistrationHandler(q *db.Queries, opaqueServer *opaque.Server) gin.HandlerFunc {
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

		credentialIdentifier, err := utils.DecodeBase64(req.CredentialID)
		if err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": err})
			return
		}

		clientIdentity := []byte(req.Username)

		// Deserialize the client's registration record
		_, err = opaqueServer.Deserialize.RegistrationRecord(registrationRecord)
		if err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": "Invalid opaque registration record"})
			return
		}

		// Save to the database
		userParams := db.InsertUserParams{
			CredentialIdentifier:         credentialIdentifier,
			ClientIdentity:               clientIdentity,
			SerializedRegistrationRecord: registrationRecord,
		}

		appUser, err := q.InsertUser(ctx, userParams)
		if err != nil {
			log.Printf("Failed to save user to database: %v", err)
			ctx.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to save registration data"})
			return
		}

		responseBody := FinalizeRegistrationResponse{
			Message:  "Registration successful",
			UserID:   appUser.ID,
			Username: string(appUser.ClientIdentity),
		}

		// Respond with success and user details
		ctx.JSON(http.StatusOK, responseBody)
	}
}

type InitializeLoginRequest struct {
	Username   string `form:"username" json:"username" xml:"username" binding:"required"`
	KE1Message string `form:"ke1_message" json:"ke1_message" xml:"ke1_message" binding:"required"`
}

type InitializeLoginResponse struct {
	KE2Message string `json:"ke2_message"`
}

func CreateInitializeLoginHandler(q *db.Queries, opaqueServer *opaque.Server) gin.HandlerFunc {
	return func(ctx *gin.Context) {
		var req InitializeLoginRequest
		if err := ctx.ShouldBindJSON(&req); err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}

		// Parse and decode KE1 message
		ke1Message, err := utils.DecodeBase64(req.KE1Message)
		if err != nil {
			log.Fatal(err)
			ctx.JSON(http.StatusBadRequest, gin.H{"error": "Invalid KE1 message"})
			return
		}

		ke1, err := opaqueServer.Deserialize.KE1(ke1Message)
		if err != nil {
			log.Fatal(err)
			ctx.JSON(http.StatusBadRequest, gin.H{"error": "Failed to deserialize KE1"})
			return
		}

		// Fetch ClientRecord from database
		appUser, err := q.GetUserByUsername(ctx, []byte(req.Username))
		if err != nil {
			log.Printf("%+v", err)
			ctx.JSON(http.StatusNotFound, gin.H{"error": "Credential not found"})
			return
		}

		// Deserialize the registration record from database
		registrationRecord, err := opaqueServer.Deserialize.RegistrationRecord(appUser.SerializedRegistrationRecord)
		if err != nil {
			log.Fatal(err)
			ctx.JSON(http.StatusBadRequest, gin.H{"error": "Cannot deserialize opaque registration record found in database."})
		}

		// Perform server-side LoginInit
		ke2, err := opaqueServer.LoginInit(
			ke1,
			&opaque.ClientRecord{
				CredentialIdentifier: appUser.CredentialIdentifier,
				ClientIdentity:       appUser.ClientIdentity,
				RegistrationRecord:   registrationRecord,
			},
			opaque.ServerLoginInitOptions{
				EphemeralSecretKey: opaque.P256Sha256.Group().HashToScalar([]byte("input"), []byte("dstlongerthan16bytes")),
			},
		)
		if err != nil {
			log.Fatal(err)
			ctx.JSON(http.StatusInternalServerError, gin.H{"error": "Login initialization failed"})
			return
		}

		ke2Message := utils.EncodeBase64(ke2.Serialize())
		responseBody := InitializeLoginResponse{
			KE2Message: ke2Message,
		}
		ctx.JSON(http.StatusOK, responseBody)
	}
}

type FinalizeLoginRequest struct {
	KE3Message string `form:"ke3_message" json:"ke3_message" xml:"ke3_message" binding:"required"`
}

type FinalizeLoginResponse struct {
	Status string `json:"status"`
}

func CreateFinalizeLoginHandler(opaqueServer *opaque.Server) gin.HandlerFunc {
	return func(ctx *gin.Context) {
		var req FinalizeLoginRequest
		if err := ctx.ShouldBindJSON(&req); err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}

		// Decode and deserialize KE3
		ke3Message, err := utils.DecodeBase64(req.KE3Message)
		if err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": "Invalid KE3 message"})
			return
		}

		ke3, err := opaqueServer.Deserialize.KE3(ke3Message)
		if err != nil {
			ctx.JSON(http.StatusBadRequest, gin.H{"error": "Failed to deserialize KE3"})
			return
		}

		// Finalize login
		if err := opaqueServer.LoginFinish(ke3); err != nil {
			ctx.JSON(http.StatusUnauthorized, gin.H{"error": "Login failed"})
			return
		}

		// Generate and send a session token (e.g., JWT)
		// sessionKey := opaqueServer.SessionKey()
		// fmt.Println(utils.EncodeBase64(sessionKey))

		responseBody := FinalizeLoginResponse{
			Status: "success",
		}

		ctx.JSON(http.StatusOK, responseBody)
	}
}

type OpaqueConfResponse struct {
	SerializedConf string `json:"serialized_conf"`
	ServerID       string `json:"server_id"`
}

func CreateGetOpaqueConfHandler(opaqueSetup *utils.OpaqueSetupType) gin.HandlerFunc {
	return func(ctx *gin.Context) {
		ctx.JSON(http.StatusOK, OpaqueConfResponse{
			SerializedConf: utils.EncodeBase64(opaqueSetup.Conf.Serialize()),
			ServerID:       utils.EncodeBase64(opaqueSetup.ServerID),
		})
	}
}
