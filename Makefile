test:
	clear
	tmux clear-history || true
	cargo --color always test

test-less:
	make test 2>&1|less -R

test-loop:
	find . | entr make test

.PHONY: test test-less test-loop
