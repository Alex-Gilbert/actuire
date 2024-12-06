.PHONY: build_tests

build_tests:
	@echo "Compiling tests..."
	@cargo test --no-run 2>&1 | tee output.txt
	@echo "Extracting test binary path..."
	# Use grep and awk to find the line mentioning the executable, extract the path, and remove parentheses
	@cat output.txt | grep "Executable" | awk '{print $$NF}' | sed 's/[()]//g' > .nvim/test_binary_path.txt
	@echo "Test binary path written to test_binary_path.txt without parentheses"
	@rm output.txt
