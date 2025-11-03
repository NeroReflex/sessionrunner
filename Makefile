# Build variables
BUILD_TYPE ?= release
TARGET ?= $(shell rustc -vV | grep "host" | sed 's/host: //')
ETC_DIR ?= etc

.PHONY_: install_sessionrunner
install_sessionrunner: target/$(TARGET)/$(BUILD_TYPE)/sessionrunner target/$(TARGET)/$(BUILD_TYPE)/sessionrunnerctl
	install -D -m 755 target/$(TARGET)/$(BUILD_TYPE)/sessionrunner $(PREFIX)/usr/bin/sessionrunner
	install -D -m 755 target/$(TARGET)/$(BUILD_TYPE)/sessionrunnerctl $(PREFIX)/usr/bin/sessionrunnerctl
	install -D -m 755 rootfs/usr/share/wayland-sessions/sessionrunner.desktop $(PREFIX)/usr/share/wayland-sessions/sessionrunner.desktop

install_start-sessionrunner: target/$(TARGET)/$(BUILD_TYPE)/start-sessionrunner
	install -D -m 755 target/$(TARGET)/$(BUILD_TYPE)/start-sessionrunner $(PREFIX)/usr/bin/start-sessionrunner

.PHONY_: install_sessionexec
install_sessionexec: target/$(TARGET)/$(BUILD_TYPE)/sessionexec
	install -D -m 755 target/$(TARGET)/$(BUILD_TYPE)/sessionexec $(PREFIX)/usr/bin/sessionexec
	install -D -m 755 rootfs/usr/lib/sessionexec/session-return.sh $(PREFIX)/usr/lib/sessionexec/session-return.sh
	install -D -m 755 rootfs/usr/lib/os-session-select $(PREFIX)/usr/lib/os-session-select
	install -D -m 644 rootfs/usr/lib/sessionrunner/steamdeck.service $(PREFIX)/usr/lib/sessionrunner/steamdeck.service
	install -D -m 644 rootfs/usr/lib/sessionrunner/default.service $(PREFIX)/usr/lib/sessionrunner/default.service
	install -D -m 755 rootfs/usr/share/wayland-sessions/game-mode.desktop $(PREFIX)/usr/share/wayland-sessions/game-mode.desktop
	install -D -m 755 rootfs/usr/share/applications/org.sessionexec.session-return.desktop $(PREFIX)/usr/share/applications/org.sessionexec.session-return.desktop
	rm -f $(PREFIX)/usr/share/wayland-sessions/default.desktop
	ln -s game-mode.desktop $(PREFIX)/usr/share/wayland-sessions/default.desktop

.PHONY: install
install: install_sessionrunner install_sessionexec install_start-sessionrunner

.PHONY: build
build: target/$(TARGET)/$(BUILD_TYPE)/sessionexec target/$(TARGET)/$(BUILD_TYPE)/sessionrunner target/$(TARGET)/$(BUILD_TYPE)/sessionrunnerctl

.PHONY: fetch
fetch: Cargo.lock
	cargo fetch --locked

target/$(TARGET)/$(BUILD_TYPE)/sessionexec: fetch
	cargo build --frozen --offline --all-features --$(BUILD_TYPE) --target=$(TARGET) --target-dir target --bin sessionexec

target/$(TARGET)/$(BUILD_TYPE)/start-sessionrunner: fetch
	cargo build --frozen --offline --all-features --$(BUILD_TYPE) --target=$(TARGET) --target-dir target --bin start-sessionrunner

target/$(TARGET)/$(BUILD_TYPE)/sessionrunner: fetch
	cargo build --frozen --offline --all-features --$(BUILD_TYPE) --target=$(TARGET) --target-dir target

target/$(TARGET)/$(BUILD_TYPE)/sessionrunnerctl: fetch
	cargo build --frozen --offline --all-features --$(BUILD_TYPE) --target=$(TARGET) --target-dir target --bin sessionrunnerctl

.PHONY: clean
clean:
	cargo clean

.PHONY: all
all: build

.PHONY: deb
deb: fetch
	cargo-deb --all-features
