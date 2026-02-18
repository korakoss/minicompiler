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
				output, err := exec.Command("ssh", "pi", "zsh -l -c 'cd /home/akos/programming_projects/minicompiler && source /home/akos/.cargo/env && cargo build'").CombinedOutput()
				println(string(output))
				if err != nil {
					println(err)
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
			programTargetDir := "yum/target/" + programName 
			commands := []exec.Cmd{
				*exec.Command("rm", "-rf", programTargetDir),
				*exec.Command("mkdir", programTargetDir),
			}
			commands = append(commands, makeRunCmds("yum/src/" + programName + ".yum", programTargetDir)...)
			output := executeCommandListOnPi(commands)
			fmt.Println(output)
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
				targetDir := "tests/target" + testName
				commands := []exec.Cmd{
					*exec.Command("rm", "-rf", targetDir),
					*exec.Command("mkdir", targetDir),
				}
				commands = append(commands, makeRunCmds("tests/src/" + testName + ".yum", targetDir)...)
				output := executeCommandListOnPi(commands)
				fmt.Println(output)
			}
			negativeTestCases := []string{"bad_branch"}
			for _, testName := range negativeTestCases {
				targetDir := "tests/target" + testName
				commands := []exec.Cmd{
					*exec.Command("rm", "-rf", targetDir),
					*exec.Command("mkdir", targetDir),
				}
				commands = append(commands, makeRunCmds("tests/src/" + testName + ".yum", targetDir)...)
				output := executeCommandListOnPi(commands)
				fmt.Println(output)
			}
        },
    }
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
	fmt.Println(cmdStrings)
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

func makeRunCmds (programSrcPath, programTargetDir string) (cmdSequence []exec.Cmd) {
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

