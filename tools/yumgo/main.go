package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/fatih/color"
	"github.com/spf13/cobra"
)

func main() {
    var noBuild bool
    var keepIR bool
    
    rootCmd := &cobra.Command{
		Use: "yum",
		Short: "Build and test tool for the Yum language",
	}
    
    rootCmd.PersistentFlags().BoolVarP(&noBuild, "no-build", "n", false, "Skip cargo")
    rootCmd.PersistentFlags().BoolVar(&keepIR, "keep-ir", false, "Keep IRs")
    
    rootCmd.AddCommand(
        makeRunCmd(&noBuild, &keepIR),
        makeTestCmd(&noBuild, &keepIR),
    )

	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}

func makeRunCmd(noBuild, keepIR *bool) *cobra.Command {
    return &cobra.Command{
        Use:   "run [program]",
        Args:  cobra.ExactArgs(1),
        Run: func(cmd *cobra.Command, args []string) {
			programName := args[0]

			if !*noBuild {
				buildCargo()
			}


        },
    }
}


func makeTestCmd(noBuild, keepIR *bool) *cobra.Command {
    return &cobra.Command{
        Use:   "test",
        Args:  cobra.NoArgs,
        Run: func(cmd *cobra.Command, args []string) {
            // Use *noBuild, *keepIR, args[0]
        },
    }
}

func buildCargo() {}

