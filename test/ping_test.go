package main

import (
	"net/http"
	"salauskilke/internal/handlers"
	"testing"

	"github.com/go-resty/resty/v2"
	"github.com/stretchr/testify/assert"
)

func TestPing(t *testing.T) {
	
    client := resty.New()
    payload := handlers.PingRequest{
		Message: "ping",
	}

    resp, err := client.R().
        SetHeader("Content-Type", "application/json").
        SetBody(payload).
		SetResult(&handlers.PingResponse{}).
        Post("http://localhost:8080/api/ping")


    assert.NoError(t, err)
    assert.Equal(t, http.StatusOK, resp.StatusCode())
	result := resp.Result().(*handlers.PingResponse)
    assert.Equal(t, "ping", result.Message)
}
