package main

import (
	"bytes"
	"testing"

	"github.com/bytemare/opaque"
	"github.com/stretchr/testify/assert"
)

var (
	exampleClientRecord                               *opaque.ClientRecord
	secretOprfSeed, serverPrivateKey, serverPublicKey []byte
)

// serverSetup sets up the long-term values for the OPAQUE server.
func serverSetup(t *testing.T) {
	serverID := []byte("server-identity")
	conf := opaque.DefaultConfiguration()
	secretOprfSeed = conf.GenerateOPRFSeed()
	serverPrivateKey, serverPublicKey = conf.KeyGen()

	if serverPrivateKey == nil || serverPublicKey == nil || secretOprfSeed == nil {
		t.Fatalf("Oh no! Something went wrong setting up the server secrets!")
	}

	server, err := conf.Server()
	if err != nil {
		t.Fatal(err)
	}

	if err := server.SetKeyMaterial(serverID, serverPrivateKey, serverPublicKey, secretOprfSeed); err != nil {
		t.Fatal(err)
	}
}

// registration demonstrates the interactions between a client and a server for the registration phase.
func registration(t *testing.T) {
	// Prepare registration data
	password := []byte("password")
	serverID := []byte("server")
	clientID := []byte("username")
	conf := opaque.DefaultConfiguration()

	client, err := conf.Client()
	if err != nil {
		t.Fatal(err)
	}

	server, err := conf.Server()
	if err != nil {
		t.Fatal(err)
	}

	var message1, message2, message3 []byte
	var credID []byte

	// Client starts registration
	c1 := client.RegistrationInit(password)
	message1 = c1.Serialize()

	// Server handles registration request
	request, err := server.Deserialize.RegistrationRequest(message1)
	if err != nil {
		t.Fatal(err)
	}
	credID = opaque.RandomBytes(64)
	pks, err := server.Deserialize.DecodeAkePublicKey(serverPublicKey)
	if err != nil {
		t.Fatal(err)
	}
	response := server.RegistrationResponse(request, pks, credID, secretOprfSeed)
	message2 = response.Serialize()

	// Client finalizes registration
	response, err = client.Deserialize.RegistrationResponse(message2)
	if err != nil {
		t.Fatal(err)
	}
	record, _ := client.RegistrationFinalize(response, opaque.ClientRegistrationFinalizeOptions{
		ClientIdentity: clientID,
		ServerIdentity: serverID,
	})
	message3 = record.Serialize()

	// Server stores the registration record
	record, err = server.Deserialize.RegistrationRecord(message3)
	if err != nil {
		t.Fatal(err)
	}

	exampleClientRecord = &opaque.ClientRecord{
		CredentialIdentifier: credID,
		ClientIdentity:       clientID,
		RegistrationRecord:   record,
	}
}

// TestOpaque runs the full OPAQUE login and authentication process to ensure it works.
func TestOpaque(t *testing.T) {
	// Step 1: Set up server and registration (executed only once)
	serverSetup(t)
	registration(t)

	// Step 2: Test login
	password := []byte("password")
	serverID := []byte("server")
	clientID := []byte("username")
	conf := opaque.DefaultConfiguration()

	client, err := conf.Client()
	if err != nil {
		t.Fatal(err)
	}

	server, err := conf.Server()
	if err != nil {
		t.Fatal(err)
	}

	if err := server.SetKeyMaterial(serverID, serverPrivateKey, serverPublicKey, secretOprfSeed); err != nil {
		t.Fatal(err)
	}

	// Step 3: The client and server exchange messages during login
	var message1, message2, message3 []byte
	var clientSessionKey, serverSessionKey []byte

	// Client sends KE1
	ke1 := client.LoginInit(password)
	message1 = ke1.Serialize()

	// Server handles KE1 and sends KE2
	ke1Deserialized, err := server.Deserialize.KE1(message1)
	if err != nil {
		t.Fatal(err)
	}

	ke2, err := server.LoginInit(ke1Deserialized, exampleClientRecord)
	if err != nil {
		t.Fatal(err)
	}
	message2 = ke2.Serialize()

	// Client handles KE2 and sends KE3
	ke2Deserialized, err := client.Deserialize.KE2(message2)
	if err != nil {
		t.Fatal(err)
	}

	ke3, _, err := client.LoginFinish(ke2Deserialized, opaque.ClientLoginFinishOptions{
		ClientIdentity: clientID,
		ServerIdentity: serverID,
	})
	if err != nil {
		t.Fatal(err)
	}
	message3 = ke3.Serialize()

	// Server handles KE3 and finalizes the session
	ke3Deserialized, err := server.Deserialize.KE3(message3)
	if err != nil {
		t.Fatal(err)
	}

	if err := server.LoginFinish(ke3Deserialized); err != nil {
		t.Fatal(err)
	}

	// Verify session keys match
	clientSessionKey = client.SessionKey()
	serverSessionKey = server.SessionKey()

	assert.True(t, bytes.Equal(clientSessionKey, serverSessionKey), "The shared session keys should match.")

	t.Log("OPAQUE is much awesome!")
}
