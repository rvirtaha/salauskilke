package main

import (
	"context"
	"fmt"
	"log"
	"os"
	router "salauskilke/internal"
	"salauskilke/internal/generated/db"
	"salauskilke/internal/utils"
	"time"
)

func main() {
	// Database connection
	connString := "postgres://salauskilke:secret@localhost:5432/salauskilke?sslmode=disable"
	timeoutDuration := 5 * time.Second

	conn, err := utils.SetupDatabase(connString, timeoutDuration)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Database setup failed: %v\n", err)
		os.Exit(1)
	}
	defer conn.Close(context.Background())
	q := db.New(conn)

	// Opaque auth setup
	opaqueSetup, err := utils.OpaqueSetup()
	if err != nil {
		log.Fatalln(err)
	}

	// Start Gin router
	router := router.SetupRouter(q, opaqueSetup)
	if err := router.Run(":8080"); err != nil {
		fmt.Fprintf(os.Stderr, "Unable to start server: %v\n", err)
		os.Exit(1)
	}
}
