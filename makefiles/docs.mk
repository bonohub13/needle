CURRENT_VERSION := $(shell grep "^version" Cargo.toml | awk '{print$$NF}')

update-docs:
	@sed -i -E "s;(download)/[^\/]*/(.*);\1/${CURRENT_VERSION}/\2;g" README.md
