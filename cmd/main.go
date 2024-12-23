package main

import (
	"context"
	"fmt"
	"os"
	router "salauskilke/internal"
	"salauskilke/internal/generated/db"
	"time"

	"github.com/jackc/pgx/v5"
)

func main() {

	// Database connection
	timeoutDuration := 5 * time.Second
	timeoutCtx, cancel := context.WithTimeout(context.Background(), timeoutDuration)
	defer cancel()

	conn, err := pgx.Connect(timeoutCtx, "postgres://salauskilke:secret@localhost:5432/salauskilke?sslmode=disable")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Unable to connect to database: %v\n", err)
		os.Exit(1)
	}
	defer conn.Close(context.Background())
	q := db.New(conn)

	

	// Start Gin router
	router := router.SetupRouter(q)
	if err := router.Run(":8080"); err != nil {
		fmt.Fprintf(os.Stderr, "Unable to start server: %v\n", err)
		os.Exit(1)
	}
}