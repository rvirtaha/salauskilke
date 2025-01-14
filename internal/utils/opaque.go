package utils

import (
	"fmt"
	"log"

	"github.com/bytemare/opaque"
)

type OpaqueSetupType struct {
	ServerID []byte
	SecretOprfSeed []byte
	ServerPrivateKey []byte
	ServerPublicKey []byte
	Conf *opaque.Configuration
}

func OpaqueSetup() (*OpaqueSetupType, error) {

	serverID := []byte("salauskilke")
	conf := opaque.DefaultConfiguration()
	secretOprfSeed := conf.GenerateOPRFSeed()
	serverPrivateKey, serverPublicKey := conf.KeyGen()

	if serverPrivateKey == nil || serverPublicKey == nil || secretOprfSeed == nil {
		log.Fatalf("Something went wrong setting up the server secrets!")
	}

	server, err := conf.Server()
	if err != nil {
		log.Fatalln(err)
		return nil, err
	}

	if err := server.SetKeyMaterial(serverID, serverPrivateKey, serverPublicKey, secretOprfSeed); err != nil {
		log.Fatalln(err)
		return nil, err
	}

	fmt.Println("OPAQUE server initialized.")

	out := OpaqueSetupType{
		ServerID: serverID,
		SecretOprfSeed: secretOprfSeed,
		ServerPrivateKey: serverPrivateKey,
		ServerPublicKey: serverPublicKey,
		Conf: conf,
	}

	return &out, nil
}