package main

import (
	"net/http"
	"salauskilke/internal/handlers"
	"salauskilke/internal/utils"
	"testing"

	"github.com/bytemare/opaque"
	"github.com/go-resty/resty/v2"
	"github.com/stretchr/testify/assert"
)


func TestRegistration(t *testing.T) {
	var (
		password []byte = []byte("password")
		serverID []byte = []byte("salauskilke")
		clientID []byte = opaque.RandomBytes(16)
		baseUrl	 string = "http://localhost:8080/api"
	)

	conf := opaque.DefaultConfiguration()

	opaqueClient, err := conf.Client()
	assert.NoError(t, err)

	httpClient := resty.New()

	// ---------- /register/initialize ----------
	registrationMessage := utils.EncodeBase64(opaqueClient.RegistrationInit(password).Serialize())

	initPayload := handlers.InitializeRegistrationRequest{
		RegistrationMessage: registrationMessage,
	}

	initResp, err := httpClient.R().
		SetHeader("Content-Type", "application/json").
		SetBody(initPayload).
		SetResult(&handlers.InitializeRegistrationResponse{}).
		Post(baseUrl + "/register/initialize")

	assert.NoError(t, err)
	assert.Equal(t, http.StatusOK, initResp.StatusCode())
	initResult := initResp.Result().(*handlers.InitializeRegistrationResponse)
	
	t.Logf("%v", initResult)

	// ---------- /register/finalize ----------
	responseMessage, err := utils.DecodeBase64(initResult.ResponseMessage); assert.NoError(t, err)
	credID, err  := utils.DecodeBase64(initResult.CredentialID); 			assert.NoError(t, err)
	
	registrationResponse, err := opaqueClient.Deserialize.RegistrationResponse(responseMessage)
	assert.NoError(t, err)
	registrationRecord, exportKey := opaqueClient.RegistrationFinalize(registrationResponse, opaque.ClientRegistrationFinalizeOptions{
		ClientIdentity: clientID,
		ServerIdentity: serverID,
	})
	serializedRecord := utils.EncodeBase64(registrationRecord.Serialize())
	serializedCredID := utils.EncodeBase64(credID)
	serializedUsername := utils.EncodeBase64(clientID)
	
	finPayload := handlers.FinalizeRegistrationRequest{
		Username: serializedUsername,
		RegistrationRecord: serializedRecord,
		CredentialID: serializedCredID,
	}

	finResp, err := httpClient.R().
		SetHeader("Content-Type", "application/json").
		SetBody(finPayload).
		SetResult(&handlers.FinalizeRegistrationResponse{}).
		Post(baseUrl + "/register/finalize")
	
	assert.NoError(t, err)
	assert.Equal(t, http.StatusOK, finResp.StatusCode())
	finResult := finResp.Result().(*handlers.FinalizeRegistrationResponse)

	t.Logf("Final result: %v", finResult)
	t.Logf("Export key: %v", exportKey)
}

func TestLogin(t *testing.T) {
	var (
		password []byte = []byte("password")
		serverID []byte = []byte("salauskilke")
		clientID []byte = opaque.RandomBytes(16)
		baseUrl	 string = "http://localhost:8080/api"
	)

	conf := opaque.DefaultConfiguration()

	opaqueClient, err := conf.Client()
	assert.NoError(t, err)

	httpClient := resty.New()

	// ---------- /login/initialize ----------

	ke1 := utils.EncodeBase64(opaqueClient.LoginInit(password).Serialize())
	username := utils.EncodeBase64(clientID)

	initPayload := handlers.InitializeLoginRequest{
		Username: username,
		KE1Message: ke1,
	}

	initResp, err := httpClient.R().
		SetHeader("Content-Type", "application/json").
		SetBody(initPayload).
		SetResult(&handlers.InitializeLoginResponse{}).
		Post(baseUrl + "/login/initialize")

	assert.NoError(t, err)
	assert.Equal(t, http.StatusOK, initResp.StatusCode())
	initResult := initResp.Result().(*handlers.InitializeLoginResponse)
	
	t.Logf("%v", initResult)

	// ---------- /login/finalize ----------

	ke2Message, err := utils.DecodeBase64(initResult.KE2Message);	assert.NoError(t, err)
	ke2, err := opaqueClient.Deserialize.KE2(ke2Message);			assert.NoError(t, err)

	ke3, exportKey, err := opaqueClient.LoginFinish(ke2, opaque.ClientLoginFinishOptions{
		ClientIdentity: clientID,
		ServerIdentity: serverID,
	})
	assert.NoError(t, err)

	ke3Serialized := utils.EncodeBase64(ke3.Serialize())

	finPayload := handlers.FinalizeLoginRequest{
		KE3Message: ke3Serialized,
	}

	finResp, err := httpClient.R().
		SetHeader("Content-Type", "application/json").
		SetBody(finPayload).
		SetResult(&handlers.FinalizeLoginResponse{}).
		Post(baseUrl + "/login/finalize")
	
	assert.NoError(t, err)
	assert.Equal(t, http.StatusOK, initResp.StatusCode())
	finResult := finResp.Result().(*handlers.FinalizeLoginResponse)
	assert.Equal(t, "success", finResult.Status)

	t.Logf("Final result: %v", finResult)
	t.Logf("Export key: %v", exportKey)
}