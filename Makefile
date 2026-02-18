.PHONY: all build clean

all: build

build: clean
	@rsync -av --exclude "target" --exclude ".git" "/Users/akoskorosi/programming_projects/yum/minicompiler/" "pi:/home/akos/programming_projects/minicompiler"	
	@cargo build	
	@cp target/debug/minicompiler ./bin/yumc
	@cd tools/yumgo && go build -o ../../bin/yum .
	@ssh pi -t "source ~/.zshrc && cd ~/programming_projects/minicompiler && cargo build && cp target/debug/minicompiler ./bin/yumc && cd tools/yumgo && go build -o ../../bin/yum ."

clean: 
	@cargo clean 
	@ssh pi -t "rm -rf /home/akos/programming_projects/minicompiler/"
	@ssh pi -t "cd /home/akos/programming_projects && mkdir minicompiler"
	@rm -f target/debug/minicompiler
	@rm -f ./bin/yumc
	@rm -f ./bin/yum
	@echo "Cleaned up"

