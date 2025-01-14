// utils/encoding.go
package utils

import (
	"encoding/base64"
	"fmt"
)

var encoder = base64.URLEncoding.WithPadding(base64.NoPadding)

// DecodeBase64 decodes an unpadded, URL-safe Base64 string.
func DecodeBase64(input string) ([]byte, error) {
	decoded, err := encoder.DecodeString(input)
	if err != nil {
		return nil, fmt.Errorf("error decoding base64: %w", err)
	}
	return decoded, nil
}

// EncodeBase64 encodes data into an unpadded, URL-safe Base64 string.
func EncodeBase64(input []byte) string {
	return encoder.EncodeToString(input)
}
