package utils

import (
	"context"
	"fmt"
	"time"

	"github.com/jackc/pgx/v5"
)

// SetupDatabase initializes and returns a new database query instance.
func SetupDatabase(connString string, timeout time.Duration) (*pgx.Conn, error) {
	timeoutCtx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	conn, err := pgx.Connect(timeoutCtx, connString)
	if err != nil {
		return nil, fmt.Errorf("unable to connect to database: %w", err)
	}

	return conn, nil
}
