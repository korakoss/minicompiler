package main

import (
	"os"
	"os/exec"
	"github.com/spf13/cobra"
	"fmt"
	"strings"
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


func executeCommandListOnPi(commands []exec.Cmd) {
	hostname, err := os.Hostname()
	if err != nil {
		os.Exit(1)
	}
	switch hostname {		
	case "pi":
		for _,cmd := range commands {
			output, err := cmd.CombinedOutput()
			fmt.Printf("%s", output)
			if err != nil {
				os.Exit(1)
			}
		}
	case "mac":
		var cmdStrings []string	
		cmdStrings = append(cmdStrings, "source ~/.zshrc")
		cmdStrings = append(cmdStrings, "cd ~/programming_projects/minicompiler")
		for _, cmd := range commands {
			var cmdString string	
			cmdString += strings.Join(cmd.Env, " ")
			cmdString += strings.Join(cmd.Args, " ")
			cmdStrings = append(cmdStrings, cmdString)
		}
		bigCmdString := strings.Join(cmdStrings, " && ")
		bigRemoteCmd := exec.Command("ssh", "pi", "-t", bigCmdString) 
		output, err := bigRemoteCmd.CombinedOutput()
		fmt.Println(string(output))
		if err != nil {
			os.Exit(1)
		}
	default:
		os.Exit(1)
	}
}

func makeRunCmds (programSrcPath, programTargetDir string) (cmdSequence []exec.Cmd) {
	// TODO: Make the corresponding API changes in Rust main	

	assPath := programTargetDir + "/asm.s"
	tempObjPath := programTargetDir + "/TEMP.o"
	exPath := programTargetDir + "/exec"

	cmdSequence = []exec.Cmd{
		*exec.Command("RUST_BACKTRACE=1", "bin/yumc", programSrcPath, programTargetDir),
		*exec.Command("as", "-o", tempObjPath, assPath),
		*exec.Command("gcc", "-o", exPath, tempObjPath),
		*exec.Command("rm", tempObjPath),
		*exec.Command(exPath),
	}
	return
}

