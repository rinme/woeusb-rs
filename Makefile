PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
POLKIT_DIR ?= /usr/share/polkit-1/actions
CARGO ?= cargo

.PHONY: all build test clean install uninstall arch-pkg completions

all: build completions

build:
	$(CARGO) build --release

completions: build
	install -d target/completions
	target/release/woeusb completions bash > target/completions/woeusb.bash
	target/release/woeusb completions bash --bin woeusb-ng > target/completions/woeusb-ng.bash
	target/release/woeusb completions fish > target/completions/woeusb.fish
	target/release/woeusb completions fish --bin woeusb-ng > target/completions/woeusb-ng.fish
	target/release/woeusb completions zsh > target/completions/_woeusb
	target/release/woeusb completions zsh --bin woeusb-ng > target/completions/_woeusb-ng

test:
	$(CARGO) test --workspace

clean:
	$(CARGO) clean
	rm -rf *.pkg.tar.zst pkg/ target/completions

install: build completions
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 target/release/woeusb $(DESTDIR)$(BINDIR)/woeusb
	install -m 755 target/release/woeusbgui $(DESTDIR)$(BINDIR)/woeusbgui
	ln -sf woeusb $(DESTDIR)$(BINDIR)/woeusb-ng
	install -d $(DESTDIR)$(DATADIR)/applications
	install -m 644 miscellaneous/com.github.woeusb.woeusbgui.desktop $(DESTDIR)$(DATADIR)/applications/
	install -d $(DESTDIR)$(DATADIR)/pixmaps
	install -m 644 src/WoeUSB/data/woeusb-logo.png $(DESTDIR)$(DATADIR)/pixmaps/woeusb-logo.png
	install -d $(DESTDIR)$(DATADIR)/icons/hicolor/512x512/apps
	install -m 644 src/WoeUSB/data/woeusb-logo.png $(DESTDIR)$(DATADIR)/icons/hicolor/512x512/apps/woeusb-logo.png
	install -d $(DESTDIR)$(POLKIT_DIR)
	install -m 644 miscellaneous/com.github.woeusb.woeusb.policy $(DESTDIR)$(POLKIT_DIR)/
	install -d $(DESTDIR)$(DATADIR)/bash-completion/completions
	install -m 644 target/completions/woeusb.bash $(DESTDIR)$(DATADIR)/bash-completion/completions/woeusb
	install -m 644 target/completions/woeusb-ng.bash $(DESTDIR)$(DATADIR)/bash-completion/completions/woeusb-ng
	install -d $(DESTDIR)$(DATADIR)/fish/vendor_completions.d
	install -m 644 target/completions/woeusb.fish $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/woeusb.fish
	install -m 644 target/completions/woeusb-ng.fish $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/woeusb-ng.fish
	install -d $(DESTDIR)$(DATADIR)/zsh/site-functions
	install -m 644 target/completions/_woeusb $(DESTDIR)$(DATADIR)/zsh/site-functions/_woeusb
	install -m 644 target/completions/_woeusb-ng $(DESTDIR)$(DATADIR)/zsh/site-functions/_woeusb-ng
	@if [ -z "$(DESTDIR)" ]; then \
		which update-desktop-database >/dev/null 2>&1 && update-desktop-database -q $(DATADIR)/applications || true; \
		which gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t -f $(DATADIR)/icons/hicolor || true; \
	fi

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/woeusb
	rm -f $(DESTDIR)$(BINDIR)/woeusbgui
	rm -f $(DESTDIR)$(BINDIR)/woeusb-ng
	rm -f $(DESTDIR)$(DATADIR)/applications/com.github.woeusb.woeusbgui.desktop
	rm -f $(DESTDIR)$(DATADIR)/pixmaps/woeusb-logo.png
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/512x512/apps/woeusb-logo.png
	rm -f $(DESTDIR)$(POLKIT_DIR)/com.github.woeusb.woeusb.policy
	rm -f $(DESTDIR)$(DATADIR)/bash-completion/completions/woeusb
	rm -f $(DESTDIR)$(DATADIR)/bash-completion/completions/woeusb-ng
	rm -f $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/woeusb.fish
	rm -f $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/woeusb-ng.fish
	rm -f $(DESTDIR)$(DATADIR)/zsh/site-functions/_woeusb
	rm -f $(DESTDIR)$(DATADIR)/zsh/site-functions/_woeusb-ng
	@if [ -z "$(DESTDIR)" ]; then \
		which update-desktop-database >/dev/null 2>&1 && update-desktop-database -q $(DATADIR)/applications || true; \
		which gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t -f $(DATADIR)/icons/hicolor || true; \
	fi

arch-pkg:
	makepkg -f
