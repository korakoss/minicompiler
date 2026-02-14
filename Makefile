.PHONY: all build clean

all: build

build:
	@cargo build	
	@cp target/debug/minicompiler ./bin/yumc
	@cd tools/yumgo && go build -o ../../bin/yum .

clean: 
	@cargo clean	
	@rm -f yumc
	@echo "Cleaned up"

