package main

import (
	"fmt"
	"os"
	"os/exec"
	"strings"

	"golang.org/x/tools/go/analysis/passes/printf"
)


func executeCommandsOnPi(commands []exec.Cmd) {
	hostname, err := os.Hostname()
	if err != nil {
		os.Exit(1)
	}
	if hostname == "pi" {
		for _,cmd := range commands {
			err := cmd.Run()
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
		//bigRemoteCmd := exec.Command("ssh", "pi", "-t", bigCmdString) 
		fmt.Printf("%s",bigCmdString)
		//err := bigRemoteCmd.Run()
		//if err != nil {
		//	os.Exit(1)
		//}
	} else {
		os.Exit(1)
	}
}


func doSyncing() {
	hostname, err := os.Hostname()
	if err != nil {
		os.Exit(1)
	}
	var macDir, piDir string
	if hostname == "mac" {
		macDir = "~/programming_projects/yum/minicompiler"
		piDir = "pi:~/programming_projects/minicompiler"
	} else if hostname == "pi" {
		macDir = "mac:~/programming_projects/yum/minicompiler"
		piDir = "~/programming_projects/minicompiler"
	} else {
		os.Exit(1)
	}
	syncCmd := exec.Command("rsync", "-av", "--exclude", "target", macDir, piDir)
	if err := syncCmd.Run(); err != nil {
		os.Exit(1)
	}
}


func makeRunCmds (programSrcPath, programTargetDir string) (cmdSequence []exec.Cmd) {
	// TODO: Make the corresponding API changes in Rust main	

	assPath := programTargetDir + "assembly.s"
	tempObjPath := programTargetDir + "TEMP.o"
	exPath := programTargetDir + "exec"

	cmdSequence = []exec.Cmd{
		*exec.Command("RUST_BACKTRACE=1", "target/debug/minicompiler", programSrcPath, programTargetDir),
		*exec.Command("as", "-o", tempObjPath, assPath),
		*exec.Command("gcc", "-o", exPath, tempObjPath),
		*exec.Command("rm", tempObjPath),
		*exec.Command(exPath),
	}
	return
}


