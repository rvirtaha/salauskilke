package utils

import (
	"bytes"
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
	Server *opaque.Server
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
		Server: server,
	}

	return &out, nil
}

func IsSameOpaqueConf(a, b *opaque.Configuration) bool {
	if a.OPRF != b.OPRF ||
		a.KDF != b.KDF ||
		a.MAC != b.MAC ||
		a.Hash != b.Hash ||
		a.KSF != b.KSF ||
		a.AKE != b.AKE {
		return false
	}

	return bytes.Equal(a.Context, b.Context)
}