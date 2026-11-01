PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
POLKIT_DIR ?= /usr/share/polkit-1/actions
CARGO ?= cargo

.PHONY: all build test clean install uninstall

all: build

build:
	$(CARGO) build --release

test:
	$(CARGO) test --workspace

clean:
	$(CARGO) clean

install: build
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 target/release/woeusb $(DESTDIR)$(BINDIR)/woeusb
	install -m 755 target/release/woeusbgui $(DESTDIR)$(BINDIR)/woeusbgui
	install -d $(DESTDIR)$(DATADIR)/applications
	install -m 644 miscellaneous/com.github.woeusb.woeusbgui.desktop $(DESTDIR)$(DATADIR)/applications/
	install -d $(DESTDIR)$(DATADIR)/pixmaps
	install -m 644 src/WoeUSB/data/woeusb-logo.png $(DESTDIR)$(DATADIR)/pixmaps/woeusb-logo.png
	install -d $(DESTDIR)$(POLKIT_DIR)
	install -m 644 miscellaneous/com.github.woeusb.woeusb.policy $(DESTDIR)$(POLKIT_DIR)/

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/woeusb
	rm -f $(DESTDIR)$(BINDIR)/woeusbgui
	rm -f $(DESTDIR)$(DATADIR)/applications/com.github.woeusb.woeusbgui.desktop
	rm -f $(DESTDIR)$(DATADIR)/pixmaps/woeusb-logo.png
	rm -f $(DESTDIR)$(POLKIT_DIR)/com.github.woeusb.woeusb.policy
