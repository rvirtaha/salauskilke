package main

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"salauskilke/internal/generated/db"
	"salauskilke/internal/handlers"
	"salauskilke/internal/utils"
	test_utils "salauskilke/test/utils"
	"testing"
	"time"

	"github.com/bytemare/opaque"
	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)


func TestAuthHandlers(t *testing.T) {
	
	// ---------- setup ----------

	var serverID 				[]byte

	var initializeRegistration  gin.HandlerFunc
	var finalizeRegistration  	gin.HandlerFunc
	var initializeLogin			gin.HandlerFunc
	var finalizeLogin			gin.HandlerFunc
	var recorder				*httptest.ResponseRecorder


	// ---------- server ----------

	{ 	// server
		connString := "postgres://salauskilke:secret@localhost:5432/salauskilke?sslmode=disable"
		timeoutDuration := 5 * time.Second
		
		conn, err := utils.SetupDatabase(connString, timeoutDuration)
		assert.NoError(t, err)
		defer conn.Close(context.Background())
		
		q := db.New(conn)
		
		opaqueSetup, err := utils.OpaqueSetup()
		assert.NoError(t, err)
		serverID = opaqueSetup.ServerID
		
		initializeRegistration = handlers.CreateInitializeRegistrationHandler(q, opaqueSetup.Server, opaqueSetup)
		finalizeRegistration = handlers.CreateFinalizeRegistrationHandler(q, opaqueSetup.Server)
		initializeLogin = handlers.CreateInitializeLoginHandler(q, opaqueSetup.Server)
		finalizeLogin = handlers.CreateFinalizeLoginHandler(opaqueSetup.Server)
	}

	
	// ---------- client ----------
	
	{

		var (
			password string = "password"
			clientID string = "username" + test_utils.RandStringRunes(8)
		)

		// register

		{
			// ----- Register step 1 -----

			conf := opaque.DefaultConfiguration()
			client, err := conf.Client()
			assert.NoError(t, err)

			initMessage := utils.EncodeBase64(client.RegistrationInit([]byte(password)).Serialize())
			initPayload := handlers.InitializeRegistrationRequest{
				RegistrationMessage: initMessage,
			}
			initBody, err := json.Marshal(initPayload)
			assert.NoError(t, err)

			// Create a request
			recorder = httptest.NewRecorder()
			initReq, err := http.NewRequest("POST", "/register/initialize", bytes.NewBuffer(initBody))
			assert.NoError(t, err)
			initReq.Header.Set("Content-Type", "application/json")	

			// Create a Gin context from the request and response recorder
			initCtx, _ := gin.CreateTestContext(recorder)
			initCtx.Request = initReq

			initializeRegistration(initCtx)

			// Assert response
			assert.Equal(t, http.StatusOK, recorder.Code)

			var initResponse handlers.InitializeRegistrationResponse
			err = json.Unmarshal(recorder.Body.Bytes(), &initResponse)
			assert.NoError(t, err)


			// ----- Register step 2 -----

			responseMessage, err := utils.DecodeBase64(initResponse.ResponseMessage); 					assert.NoError(t, err)
			credID, err  := utils.DecodeBase64(initResponse.CredentialID); 								assert.NoError(t, err)
			registrationResponse, err := client.Deserialize.RegistrationResponse(responseMessage); 	assert.NoError(t, err)

			// registrationRecord, exportKey := ...
			registrationRecord, _ := client.RegistrationFinalize(registrationResponse, opaque.ClientRegistrationFinalizeOptions{
				ClientIdentity: []byte(clientID),
				ServerIdentity: serverID,
			})
			serializedRecord := utils.EncodeBase64(registrationRecord.Serialize())
			serializedCredID := utils.EncodeBase64(credID)

			finPayload := handlers.FinalizeRegistrationRequest{
				Username: clientID,
				RegistrationRecord: serializedRecord,
				CredentialID: serializedCredID,
			}
			finBody, err := json.Marshal(finPayload)
			assert.NoError(t, err)

			// Create a request
			recorder = httptest.NewRecorder()
			finReq, err := http.NewRequest("POST", "/register/finalize", bytes.NewBuffer(finBody))
			assert.NoError(t, err)
			finReq.Header.Set("Content-Type", "application/json")

			// Create a Gin context from the request and response recorder
			finCtx, _ := gin.CreateTestContext(recorder)
			finCtx.Request = finReq

			finalizeRegistration(finCtx)

			// Assert response
			assert.Equal(t, http.StatusOK, recorder.Code)

			var finResponse handlers.FinalizeRegistrationResponse
			err = json.Unmarshal(recorder.Body.Bytes(), &finResponse)
			assert.NoError(t, err)

			t.Log(finResponse)

		}



		// login

		{
			// ----- Login step 1 -----

			conf := opaque.DefaultConfiguration()
			client, err := conf.Client()
			assert.NoError(t, err)

			ke1 := utils.EncodeBase64(client.LoginInit([]byte(password)).Serialize())
			initPayload := handlers.InitializeLoginRequest{
				Username: clientID,
				KE1Message: ke1,
			}
			initBody, err := json.Marshal(initPayload)
			assert.NoError(t, err)

			// Create a request
			recorder = httptest.NewRecorder()
			initReq, err := http.NewRequest("POST", "/login/initialize", bytes.NewBuffer(initBody))
			assert.NoError(t, err)
			initReq.Header.Set("Content-Type", "application/json")	

			// Create a Gin context from the request and response recorder
			initCtx, _ := gin.CreateTestContext(recorder)
			initCtx.Request = initReq

			initializeLogin(initCtx)

			// Assert response
			assert.Equal(t, http.StatusOK, recorder.Code)

			var initResponse handlers.InitializeLoginResponse
			err = json.Unmarshal(recorder.Body.Bytes(), &initResponse)
			assert.NoError(t, err)

			// ----- Login step 2 -----

			ke2Message, err := utils.DecodeBase64(initResponse.KE2Message)
			assert.NoError(t, err)
			ke2, err := client.Deserialize.KE2(ke2Message)
			assert.NoError(t, err)

			// ke3, exportKey := ...
			ke3, _, err := client.LoginFinish(ke2, opaque.ClientLoginFinishOptions{
				ClientIdentity: []byte(clientID),
				ServerIdentity: serverID,
			})
			assert.NoError(t, err)

			finPayload := handlers.FinalizeLoginRequest{
				KE3Message: utils.EncodeBase64(ke3.Serialize()),
			}
			finBody, err := json.Marshal(finPayload)
			assert.NoError(t, err)

			// Create a request
			recorder = httptest.NewRecorder()
			finReq, err := http.NewRequest("POST", "/login/finalize", bytes.NewBuffer(finBody))
			assert.NoError(t, err)
			initReq.Header.Set("Content-Type", "application/json")	

			// Create a Gin context from the request and response recorder
			finCtx, _ := gin.CreateTestContext(recorder)
			finCtx.Request = finReq

			finalizeLogin(finCtx)

			// Assert response
			assert.Equal(t, http.StatusOK, recorder.Code)

			var finResponse handlers.FinalizeLoginResponse
			err = json.Unmarshal(recorder.Body.Bytes(), &finResponse)
			assert.NoError(t, err)

			t.Log(finResponse)

		}

	}

}

