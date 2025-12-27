package cmd

import (
	"github.com/spf13/cobra"
	"github.com/studio713/jaxon/internal"
)

var minimal bool

var initCmd = &cobra.Command{
	Use:   "init",
	Short: "Initialize jaxon",
	Long: `Initialize a new jaxon project in the current directory.

This command creates the base configuration files required for jaxon
to operate. By default, it generates:

  - jaxon.toml        Project configuration file
  - products.json    Initial product definition file

You can use the --minimal flag to generate only the configuration file
and skip creating product-related files.`,
	Example: `  # Initialize a full jaxon project
  jaxon init

  # Initialize only jaxon.toml
  jaxon init --minimal`,
	Run: func(cmd *cobra.Command, args []string) {
		internal.InitToml()
		if !minimal {
			internal.InitProductJson()
		}
	},
}

func init() {
	rootCmd.AddCommand(initCmd)

	initCmd.Flags().BoolVarP(&minimal, "minimal", "m", false, "Create only the toml file")

}
