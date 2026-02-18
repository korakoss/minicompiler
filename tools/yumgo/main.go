package main

import (
	"os"
	"os/exec"
	"github.com/spf13/cobra"
)

// TODO: verbose flag
func main() {
    var noBuild bool
    rootCmd := &cobra.Command{
		Use: "yum",
		Short: "Build and test tool for the Yum language",
		PersistentPreRun: func(cmd *cobra.Command, args []string) {
			if !noBuild {
				buildCommand := []exec.Cmd{*exec.Command("cargo", "build")}
				executeCommandListOnPi(buildCommand)
			}
		},
	}
    rootCmd.PersistentFlags().BoolVarP(&noBuild, "no-build", "b", false, "Skip cargo")
    rootCmd.AddCommand(
        makeCobraRunCmd(),
        makeCobraTestCmd(),
    )
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}

func makeCobraRunCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "run [program]",
        Args:  cobra.ExactArgs(1),
        Run: func(cmd *cobra.Command, args []string) {
			programName := args[0]
			programSrcPath := "yum/src/" + programName + ".yum"
			programTargetDir := "yum/target/" + programName
			//onMac := amOnMac()
			commands := []exec.Cmd{
				*exec.Command("rm", "-rf", programTargetDir),
				*exec.Command("pwd"),
				*exec.Command("mkdir", programTargetDir),
			}
			commands = append(commands, makeRunCmds(programSrcPath, programTargetDir)...)
			executeCommandListOnPi(commands)
        },
    }
}

func makeCobraTestCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "test",
        Args:  cobra.NoArgs,
        Run: func(cmd *cobra.Command, args []string) {
			// TODO: keeping the colored prints would be cool
				
			positiveTestCases := []string{"primetest", "nonparam_func", "long_ass_binop"}
			for _, testName := range positiveTestCases {
				runCmds := makeRunCmds("tests/src/" + testName, "tests/target" + testName)
				executeCommandListOnPi(runCmds)
				// Check stdout
			}
			negativeTestCases := []string{"bad_branch"}
			for _, testName := range negativeTestCases {
				runCmds := makeRunCmds("tests/src/" + testName, "tests/target" + testName)
				executeCommandListOnPi(runCmds)
				// Check stdout
			}
        },
    }
}
