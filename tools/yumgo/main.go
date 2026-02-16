package main

import (
	"os"
	"os/exec"
	"github.com/spf13/cobra"
)

func main() {
    var noBuild bool
    var keepIR bool
	var noSync bool

    rootCmd := &cobra.Command{
		Use: "yum",
		Short: "Build and test tool for the Yum language",
		PersistentPreRun: func(cmd *cobra.Command, args []string) {
			if !noSync {
				doSyncing()
			}
		},
	}
   	
    rootCmd.PersistentFlags().BoolVarP(&noBuild, "no-build", "b", false, "Skip cargo")
    rootCmd.PersistentFlags().BoolVarP(&keepIR, "keep-ir", "i",false, "Keep IRs")
	rootCmd.PersistentFlags().BoolVarP(&noSync, "no-sync", "s",false, "Don't sync")
    
    rootCmd.AddCommand(
        makeCobraRunCmd(&noBuild, &keepIR),
        makeCobraTestCmd(&noBuild, &keepIR),
    )

	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}

func makeCobraRunCmd(noBuild, keepIR *bool) *cobra.Command {
    return &cobra.Command{
        Use:   "run [program]",
        Args:  cobra.ExactArgs(1),
        Run: func(cmd *cobra.Command, args []string) {
			programName := args[0]
	
			programSrcPath := "yum/src/" + programName
			programTargetDir := "yum/target/" + programName

			commands := []exec.Cmd{*exec.Command("rm", "-rf", programTargetDir)}

			if !*noBuild {
				commands = append(commands, *exec.Command("cargo", "build"))
			}
			commands = append(commands, makeRunCmds(programSrcPath, programTargetDir)...)
			executeCommandsOnPi(commands)
        },
    }
}

func makeCobraTestCmd(noBuild, keepIR *bool) *cobra.Command {
    return &cobra.Command{
        Use:   "test",
        Args:  cobra.NoArgs,
        Run: func(cmd *cobra.Command, args []string) {
			// TODO: keeping the colored prints would be cool
			if !*noBuild {
				executeCommandsOnPi([]exec.Cmd{*exec.Command("cargo", "build")})
			}
				
			positiveTestCases := []string{"primetest", "nonparam_func", "long_ass_binop"}
			for _, testName := range positiveTestCases {
				runCmds := makeRunCmds("tests/src/" + testName, "tests/target" + testName)
				executeCommandsOnPi(runCmds)
				// Check stdout
			}
			negativeTestCases := []string{"bad_branch"}
			for _, testName := range negativeTestCases {
				runCmds := makeRunCmds("tests/src/" + testName, "tests/target" + testName)
				executeCommandsOnPi(runCmds)
				// Check stdout
			}
        },
    }
}
