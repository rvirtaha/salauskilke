package main

import (
	"bytes"
	"net/http"
	"salauskilke/internal/handlers"
	"salauskilke/internal/utils"
	test_utils "salauskilke/test/utils"
	"testing"

	"github.com/bytemare/opaque"
	"github.com/go-resty/resty/v2"
	"github.com/stretchr/testify/assert"
)

func TestOpaqueConf(t *testing.T) {
	baseUrl	 := "http://localhost:8080/api"
	httpClient := resty.New()

	clientConf := opaque.DefaultConfiguration()

	confResp, err := httpClient.R().
		SetResult(&handlers.OpaqueConfResponse{}).
		Get(baseUrl + "/opaqueconf")
	
	assert.NoError(t, err)
	assert.Equal(t, http.StatusOK, confResp.StatusCode())

	confResult := confResp.Result().(*handlers.OpaqueConfResponse)

	resultServerID, err := utils.DecodeBase64(confResult.ServerID); assert.NoError(t, err)
	resultSerializedConf, err := utils.DecodeBase64(confResult.SerializedConf); assert.NoError(t, err)

	assert.Equal(t, resultServerID, []byte("salauskilke"))
	serverConf, err := opaque.DeserializeConfiguration(resultSerializedConf); assert.NoError(t, err)

	assert.True(t, utils.IsSameOpaqueConf(clientConf, serverConf))

}

func TestAuthentication(t *testing.T) {

	// ---------- setup ----------

	serverID := []byte("salauskilke")
	baseUrl	 := "http://localhost:8080/api"

	// ---------- client ----------
	
	{

		var (
			password string = "password"
			clientID string = "username" + test_utils.RandStringRunes(8)
			registrationExportKey []byte
			loginExportKey 		  []byte
		)

		httpClient := resty.New()

		// register

		{
			// ----- Register -----

			conf := opaque.DefaultConfiguration()
			client, err := conf.Client()
			assert.NoError(t, err)

			initMessage := utils.EncodeBase64(client.RegistrationInit([]byte(password)).Serialize())
			initPayload := handlers.InitializeRegistrationRequest{
				RegistrationMessage: initMessage,
			}
		
			initResp, err := httpClient.R().
				SetHeader("Content-Type", "application/json").
				SetBody(initPayload).
				SetResult(&handlers.InitializeRegistrationResponse{}).
				Post(baseUrl + "/register/initialize")
		
			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, initResp.StatusCode())
			initResult := initResp.Result().(*handlers.InitializeRegistrationResponse)

			// ----- Register step 2 -----

			responseMessage, err := utils.DecodeBase64(initResult.ResponseMessage); 				assert.NoError(t, err)
			credID, err  := utils.DecodeBase64(initResult.CredentialID); 							assert.NoError(t, err)
			registrationResponse, err := client.Deserialize.RegistrationResponse(responseMessage); 	assert.NoError(t, err)

			registrationRecord, exportKey := client.RegistrationFinalize(registrationResponse, opaque.ClientRegistrationFinalizeOptions{
				ClientIdentity: []byte(clientID),
				ServerIdentity: serverID,
			})
			serializedRecord := utils.EncodeBase64(registrationRecord.Serialize())
			serializedCredID := utils.EncodeBase64(credID)
			registrationExportKey = exportKey

			finPayload := handlers.FinalizeRegistrationRequest{
				Username: clientID,
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

			t.Logf("FinalizeRegistrationResponse:\n\t%+v", finResult)

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
			
			initResp, err := httpClient.R().
				SetHeader("Content-Type", "application/json").
				SetBody(initPayload).
				SetResult(&handlers.InitializeLoginResponse{}).
				Post(baseUrl + "/login/initialize")

			assert.NoError(t, err)
			assert.Equal(t, http.StatusOK, initResp.StatusCode())
			initResult := initResp.Result().(*handlers.InitializeLoginResponse)

			// ----- Login step 2 -----

			ke2Message, err := utils.DecodeBase64(initResult.KE2Message);	
			assert.NoError(t, err)
			ke2, err := client.Deserialize.KE2(ke2Message);			
			assert.NoError(t, err)

			ke3, exportKey, err := client.LoginFinish(ke2, opaque.ClientLoginFinishOptions{
				ClientIdentity: []byte(clientID),
				ServerIdentity: serverID,
			})
			assert.NoError(t, err)
			loginExportKey = exportKey

			finPayload := handlers.FinalizeLoginRequest{
				KE3Message: utils.EncodeBase64(ke3.Serialize()),
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
		
			assert.True(t, bytes.Equal(loginExportKey, registrationExportKey), "The export keys should match.")
			t.Logf("FinalizeLoginResponse:\n\t%+v", finResult)
		
		}
	}
}
