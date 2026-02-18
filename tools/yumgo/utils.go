package main

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
	"log"
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

func doSyncing() {
	fmt.Println("Syncing..")
	onMac := amOnMac()
	macDir := absPathMac(&onMac, "")
	piDir := absPathPi(&onMac, "")
	syncCmd := exec.Command("rsync", "-av", macDir, piDir)
	fmt.Println(syncCmd)
	output, err := syncCmd.CombinedOutput()
	if err != nil {
		log.Printf("rsync failed: %v\nOutput: %s", err, output)
		os.Exit(1)
	}
}

func amOnMac() (onMac bool) {
	hostname, err := os.Hostname()
	if err != nil {
		os.Exit(1)
	}
	switch hostname {
	case "mac":
		onMac = true	
	case "pi":
		onMac = false	
	default:
		fmt.Println("Unrecognized hostname")
		os.Exit(1)
	}
	return
}

func absPathPi(fromMac *bool, path string) (piPath string) {
	piPath = "/home/akos/programming_projects/minicompiler/" + path
	if *fromMac {
		piPath = "pir:" + piPath
	}
	return
}
	

func absPathMac(fromMac *bool, path string) (macPath string) {
	macPath = "/Users/akoskorosi/programming_projects/yum/minicompiler/" + path
	if !*fromMac {
		macPath = "mac:" + macPath
	}
	return
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

