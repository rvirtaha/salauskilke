package main

import (
	"bytes"
	"salauskilke/internal/utils"
	"testing"

	"github.com/bytemare/opaque"
	"github.com/stretchr/testify/assert"
)


func TestAuthLogic(t *testing.T) {
	
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

	var registrationMessage []byte
	{
		password := "password"

		registrationMessage = client.RegistrationInit([]byte(password)).Serialize()
	}

	// ---------- server ----------

	var registrationResponse []byte
	var credID []byte
	{
		request, err := server.Deserialize.RegistrationRequest(registrationMessage)
		assert.NoError(t, err)
		
		credID = opaque.RandomBytes(64)
		pks, err := server.Deserialize.DecodeAkePublicKey(opaqueSetup.ServerPublicKey)
		assert.NoError(t, err)

		registrationResponse = server.
			RegistrationResponse(request, pks, credID, opaqueSetup.SecretOprfSeed).
			Serialize()
	}

	// ---------- client ----------
	var registrationRecord []byte

	{
		response, err := client.Deserialize.RegistrationResponse(registrationResponse)
		assert.NoError(t, err)

		record, _ := client.RegistrationFinalize(response, opaque.ClientRegistrationFinalizeOptions{
			ClientIdentity: []byte(clientID),
			ServerIdentity: serverID,
		})

		registrationRecord = record.Serialize()
	}

	// ---------- server ----------
	
	{
		record, err := server.Deserialize.RegistrationRecord(registrationRecord)
		assert.NoError(t, err)

		clientRecord = opaque.ClientRecord{
			CredentialIdentifier: credID,
			ClientIdentity: []byte(clientID),
			RegistrationRecord: record,
		}
	}

	// ---------- ---------- login ---------- ----------


	// ---------- client ----------
	
	var message1 []byte
	{
		password := "password"

		ke1 := client.LoginInit([]byte(password))
		message1 = ke1.Serialize()
		
	}

	// ---------- server ----------
	
	var message2 []byte
	{
		ke1Deserialized, err := server.Deserialize.KE1(message1)
		assert.NoError(t, err)

		ke2, err := server.LoginInit(ke1Deserialized, &clientRecord)
		assert.NoError(t, err)

		message2 = ke2.Serialize()
	}

	// ---------- client ----------
	
	var message3 []byte
	{
		ke2Deserialized, err := client.Deserialize.KE2(message2)
		assert.NoError(t, err)

		ke3, _, err := client.LoginFinish(ke2Deserialized, opaque.ClientLoginFinishOptions{
			ClientIdentity: []byte(clientID),
			ServerIdentity: serverID,
		})
		assert.NoError(t, err)

		message3 = ke3.Serialize()
	}

	// ---------- server ----------
	
	{
		ke3Deserialized, err := server.Deserialize.KE3(message3)
		assert.NoError(t, err)

		if err := server.LoginFinish(ke3Deserialized); err != nil {
			assert.NoError(t, err)
		}

	}

	// Verify session keys match
	clientSessionKey := client.SessionKey()
	serverSessionKey := server.SessionKey()

	assert.True(t, bytes.Equal(clientSessionKey, serverSessionKey), "The shared session keys should match.")

	t.Log("Client and server have the same session keys!")
}

