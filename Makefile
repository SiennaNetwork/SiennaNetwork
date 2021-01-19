.PHONY: test
test:
	clear
	tmux clear-history || true
	cargo --color always test 2>&1|less -R

test-loop:
	find . | entr make test
