package cmd

import (
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:   "jaxon",
	Short: "A CLI for managing developer products and game passes",
	Long:  `jaxon is a command-line tool for creating, syncing, and managing developer products and game passes.`,
	CompletionOptions: cobra.CompletionOptions{
		// HiddenDefaultCmd: true, // hides cmd
		DisableDefaultCmd: true, // removes cmd
	},
}

func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
}
