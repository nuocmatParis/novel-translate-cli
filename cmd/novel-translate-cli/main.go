package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"

	"github.com/voocel/ainovel-cli/internal/translation/cli"
)

var version = "dev"
var commit = "unknown"
var date = "unknown"

func main() {
	if len(os.Args) == 2 && os.Args[1] == "--version" {
		fmt.Printf("novel-translate-cli %s (%s, %s)\n", version, commit, date)
		return
	}
	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt)
	defer cancel()
	os.Exit(cli.Run(ctx, os.Args[1:], os.Stdout, os.Stderr, cli.Options{}))
}
