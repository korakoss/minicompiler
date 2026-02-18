package main

import (
	"os"
	"os/exec"
	"github.com/spf13/cobra"
	"fmt"
	"strings"
)

// TODO: using some lib for path handling
// TODO: colored prints
// TODO: verbose flag
func main() {
    var noBuild bool
    rootCmd := &cobra.Command{
		Use: "yum",
		Short: "Build and test tool for the Yum language",
		PersistentPreRun: func(cmd *cobra.Command, args []string) {
			if !noBuild {
				output, err := exec.Command("ssh", "pi", "zsh -l -c 'cd /home/akos/programming_projects/minicompiler && source /home/akos/.cargo/env && cargo build'").CombinedOutput()
				println(string(output))
				if err != nil {
					os.Exit(1)
				}
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
			output := runYumProgram(programName, "yum/src/", "yum/target")
			fmt.Println(output)
        },
    }
}

func makeCobraTestCmd() *cobra.Command {
    return &cobra.Command{
        Use:   "test",
        Args:  cobra.NoArgs,
        Run: func(cmd *cobra.Command, args []string) {
			positiveTestCases := []string{"primetest", "nonparam_func", "long_ass_binop"}
			for _, testName := range positiveTestCases {
				output := runYumProgram(testName, "tests/src/", "tests/target")
				fmt.Println(output)
			}
			negativeTestCases := []string{"bad_branch"}
			for _, testName := range negativeTestCases {
				output := runYumProgram(testName, "tests/src/", "tests/target")
				fmt.Println(output)
			}
        },
    }
}

func runYumProgram(programName, srcRoot, targetRoot string) (stdOutput string) {
	sourcePath := srcRoot + programName + ".yum"
	targetDir := targetRoot + programName

	asmPath := targetDir + "/asm.s"
	objPath := targetDir + "/TEMP.o"
	exePath := targetDir + "/exec"

	cmdSequence := []exec.Cmd {
		*exec.Command("rm", "-rf", targetDir),
		*exec.Command("mkdir", targetDir),
		*exec.Command("RUST_BACKTRACE=1", "bin/yumc", sourcePath, targetDir),	// TODO: backtrace=1 should be dropped
		*exec.Command("as", "-o", objPath, asmPath),
		*exec.Command("gcc", "-o", exePath, objPath),
		*exec.Command("rm", objPath),	// TODO: could be removed from here
		*exec.Command(exePath),
	}
	stdOutput = executeCommandListOnPi(cmdSequence)
	return
}

func executeCommandListOnPi(commands []exec.Cmd) (cmdOutput string){
	var cmdStrings []string	
	for _, cmd := range commands {
		cmdString := strings.Join(cmd.Args, " ") 
		cmdStrings = append(cmdStrings, cmdString)
	}
	cmdStrings = append([]string{"cd /home/akos/programming_projects/minicompiler"}, cmdStrings...)
	cmdStrings = append([]string{"pwd"}, cmdStrings...)
	hostname, err := os.Hostname()
	if err != nil {
		os.Exit(1)
	}
	switch hostname {		
	case "pi":
		bigRemoteCmd := exec.Command("sh", "-c", strings.Join(cmdStrings, " && "))
		output, err := bigRemoteCmd.CombinedOutput()
		fmt.Println(string(output))
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
	case "mac":
		bigRemoteCmd := exec.Command("ssh", "pi", "zsh", "-l", "-c", strings.Join(cmdStrings, " && "))
		output, err := bigRemoteCmd.CombinedOutput()
		cmdOutput = string(output)
		if err != nil {
			fmt.Println("Execution error: %s", err)
			os.Exit(1)
		}
	default:
		os.Exit(1)
	}
	return
}



