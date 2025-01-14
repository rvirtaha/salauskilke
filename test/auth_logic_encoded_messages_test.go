package main

import (
	"bytes"
	"salauskilke/internal/utils"
	"testing"

	"github.com/bytemare/opaque"
	"github.com/stretchr/testify/assert"
)


func TestAuthLogicEncodedMessages(t *testing.T) {
	
	// ---------- ---------- setup ---------- ----------

	var (
		serverID []byte
		clientID = "username"
		clientRecord opaque.ClientRecord
	)
	
	conf := opaque.DefaultConfiguration()

	client, err := conf.Client()
	assert.NoError(t, err)

	opaqueSetup, err := utils.OpaqueSetup()
	assert.NoError(t, err)
		
	serverID = opaqueSetup.ServerID
	server := opaqueSetup.Server

	// ---------- ---------- register ---------- ----------

	// ---------- client ----------

	var registrationMessage string
	{
		password := "password"

		registrationMessage = utils.EncodeBase64(client.RegistrationInit([]byte(password)).Serialize())
	}

	// ---------- server ----------

	var registrationResponse string
	var credID []byte
	{
		regisrationMessageBytes, err := utils.DecodeBase64(registrationMessage)
		assert.NoError(t, err)
		request, err := server.Deserialize.RegistrationRequest(regisrationMessageBytes)
		assert.NoError(t, err)
		
		credID = opaque.RandomBytes(64)
		pks, err := server.Deserialize.DecodeAkePublicKey(opaqueSetup.ServerPublicKey)
		assert.NoError(t, err)

		registrationResponse = utils.EncodeBase64(server.RegistrationResponse(request, pks, credID, opaqueSetup.SecretOprfSeed).Serialize())
	}

	// ---------- client ----------
	var registrationRecord string
	var clientRegistrationExportKey []byte

	{
		registrationResponseBytes, err := utils.DecodeBase64(registrationResponse)
		assert.NoError(t, err)
		response, err := client.Deserialize.RegistrationResponse(registrationResponseBytes)
		assert.NoError(t, err)

		record, exportKey := client.RegistrationFinalize(response, opaque.ClientRegistrationFinalizeOptions{
			ClientIdentity: []byte(clientID),
			ServerIdentity: serverID,
		})

		registrationRecord = utils.EncodeBase64(record.Serialize())
		clientRegistrationExportKey = exportKey
	}

	// ---------- server ----------
	
	{
		registrationRecordBytes, err := utils.DecodeBase64(registrationRecord)
		assert.NoError(t, err)
		record, err := server.Deserialize.RegistrationRecord(registrationRecordBytes)
		assert.NoError(t, err)

		clientRecord = opaque.ClientRecord{
			CredentialIdentifier: credID,
			ClientIdentity: []byte(clientID),
			RegistrationRecord: record,
		}
	}

	// ---------- ---------- login ---------- ----------


	// ---------- client ----------
	
	var message1 string
	{
		password := "password"

		ke1 := client.LoginInit([]byte(password))
		message1 = utils.EncodeBase64(ke1.Serialize())
		
	}

	// ---------- server ----------
	
	var message2 string
	{
		message1bytes, err := utils.DecodeBase64(message1)
		assert.NoError(t, err)
		ke1Deserialized, err := server.Deserialize.KE1(message1bytes)
		assert.NoError(t, err)

		ke2, err := server.LoginInit(ke1Deserialized, &clientRecord)
		assert.NoError(t, err)

		message2 = utils.EncodeBase64(ke2.Serialize())
	}

	// ---------- client ----------
	
	var message3 string
	var clientLoginExportKey []byte
	{
		message2bytes, err := utils.DecodeBase64(message2)
		assert.NoError(t, err)
		ke2Deserialized, err := client.Deserialize.KE2(message2bytes)
		assert.NoError(t, err)

		ke3, exportKey, err := client.LoginFinish(ke2Deserialized, opaque.ClientLoginFinishOptions{
			ClientIdentity: []byte(clientID),
			ServerIdentity: serverID,
		})
		assert.NoError(t, err)

		message3 = utils.EncodeBase64(ke3.Serialize())
		clientLoginExportKey = exportKey
	}

	// ---------- server ----------
	
	{
		message3bytes, err := utils.DecodeBase64(message3)
		assert.NoError(t, err)
		ke3Deserialized, err := server.Deserialize.KE3(message3bytes)
		assert.NoError(t, err)

		if err := server.LoginFinish(ke3Deserialized); err != nil {
			assert.NoError(t, err)
		}

	}

	// Verify session keys match
	clientSessionKey := client.SessionKey()
	serverSessionKey := server.SessionKey()

	assert.True(t, bytes.Equal(clientSessionKey, serverSessionKey), "The shared session keys should match.")
	t.Log("Client and server have the same session keys.")

	// Verify registration and login export keys are the same
	assert.True(t, bytes.Equal(clientLoginExportKey, clientRegistrationExportKey), "The export keys should match.")
	t.Log("Client has the same export keys in login and registration.")

}

