.PHONY: dev release

VERSION ?=

dev:
	pnpm tauri dev

release:
	@set -eu; \
	branch="$$(git branch --show-current)"; \
	if [ "$$branch" != "main" ]; then \
		echo "Release must be run from the main branch (current: $$branch)."; \
		exit 1; \
	fi; \
	if [ -n "$$(git status --porcelain)" ]; then \
		echo "Release requires a clean worktree."; \
		exit 1; \
	fi; \
	git fetch origin main --tags; \
	if [ "$$(git rev-parse HEAD)" != "$$(git rev-parse origin/main)" ]; then \
		echo "Local main must match origin/main before releasing."; \
		exit 1; \
	fi; \
	version="$(VERSION)"; \
	if [ -z "$$version" ]; then \
		latest_tag="$$(git tag --list 'v[0-9]*' --sort=-version:refname | head -n 1)"; \
		if [ -z "$$latest_tag" ]; then \
			echo "No version tag found. Run make release VERSION=x.y.z."; \
			exit 1; \
		fi; \
		version="$$(printf '%s\n' "$${latest_tag#v}" | awk -F. '{ printf "%d.%d.%d", $$1, $$2, $$3 + 1 }')"; \
	fi; \
	case "$$version" in v*) tag="$$version" ;; *) tag="v$$version" ;; esac; \
	if ! printf '%s\n' "$$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$$'; then \
		echo "Invalid release version: $$version"; \
		exit 1; \
	fi; \
	echo "Creating GitHub release $$tag. This starts the Release workflow."; \
	gh release create "$$tag" --target main --title "EasyKpf $$tag" --generate-notes
