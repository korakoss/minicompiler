package main

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
)


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

func executeCommandOnPi(command exec.Cmd) {
	hostname, err := os.Hostname()
	if err != nil {
		os.Exit(1)
	}
	switch hostname {		
	case "pi":
		output, err := command.CombinedOutput()
		fmt.Printf("%s", output)
		if err != nil {
			os.Exit(1)
		}
	case "mac":
		commandString := strings.Join(command.Env, " ") + " " + strings.Join(command.Args, " ")
		commandString = "source ~/zshrc && cd/programming_projects/minicompiler && " + commandString 
		remoteCmd := exec.Command("ssh", "pi", "-t", commandString) 
		output, err := remoteCmd.CombinedOutput()
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

