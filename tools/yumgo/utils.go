package main

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
	"log"
)


func executeCommandsOnPi(commands []exec.Cmd) {
	hostname, err := os.Hostname()
	if err != nil {
		os.Exit(1)
	}
	if hostname == "pi" {
		for _,cmd := range commands {
			output, err := cmd.CombinedOutput()
			fmt.Printf("%s", output)
			if err != nil {
				os.Exit(1)
			}
		}
	} else if hostname == "mac" {
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
		fmt.Printf("%s", output)
		if err != nil {
			os.Exit(1)
		}
	} else {
		os.Exit(1)
	}
}


func doSyncing() {
	fmt.Println("Syncing..")
	hostname, err := os.Hostname()
	if err != nil {
		os.Exit(1)
	}
	macPath := "/Users/akoskorosi/programming_projects/yum/minicompiler"
	piPath := "/home/akos/programming_projects/minicompiler"
	var macDir, piDir string
	if hostname == "mac" {
		macDir = macPath
		piDir = "pi:" + piPath
	} else if hostname == "pi" {
		macDir = "mac:" + macPath
		piDir = piPath
	} else {
		fmt.Println("Unrecognized hostname")
		os.Exit(1)
	}
	syncCmd := exec.Command("rsync", "-vv", "--exclude", "target", macDir, piDir)
	output, err := syncCmd.CombinedOutput()
	if err != nil {
		log.Printf("rsync failed: %v\nOutput: %s", err, output)
		os.Exit(1)
	}
}


func makeRunCmds (programSrcPath, programTargetDir string) (cmdSequence []exec.Cmd) {
	// TODO: Make the corresponding API changes in Rust main	

	assPath := programTargetDir + "/assembly.s"
	tempObjPath := programTargetDir + "/TEMP.o"
	exPath := programTargetDir + "/exec"

	cmdSequence = []exec.Cmd{
		*exec.Command("RUST_BACKTRACE=1", "target/debug/minicompiler", programSrcPath, programTargetDir),
		*exec.Command("as", "-o", tempObjPath, assPath),
		*exec.Command("gcc", "-o", exPath, tempObjPath),
		*exec.Command("rm", tempObjPath),
		*exec.Command(exPath),
	}
	return
}

