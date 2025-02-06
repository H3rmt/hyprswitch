# Changelog

## [3.3.2](https://github.com/H3rmt/hyprswitch/compare/v3.3.1...v3.3.2) (2025-02-06)


### Bug Fixes

* fix memory leak https://github.com/H3rmt/hyprswitch/discussions/137 ([c63ff80](https://github.com/H3rmt/hyprswitch/commit/c63ff80087918395065bd80865a5e4fad217889d))
* fix release pr creation ([21894b2](https://github.com/H3rmt/hyprswitch/commit/21894b291fe433452f2ad424dd3f27b822afd7fe))

## [3.3.1](https://github.com/H3rmt/hyprswitch/compare/v3.3.0...v3.3.1) (2025-02-02)


### Bug Fixes

* **deps:** update rust crate notify-rust to v4.11.4 ([66ce579](https://github.com/H3rmt/hyprswitch/commit/66ce579d7eec3c116303faddbc07833d5869e2cf))
* **deps:** update rust crate serde_json to v1.0.138 ([142c193](https://github.com/H3rmt/hyprswitch/commit/142c1934544a46bc06f8d27695515d310a80f113))
* launcher not showing icons with absolute paths ([2b93db5](https://github.com/H3rmt/hyprswitch/commit/2b93db5281f49214defc75f7c513cf4223250545))

## [3.3.0](https://github.com/H3rmt/hyprswitch/compare/v3.2.5...v3.3.0) (2025-02-02)


### Features

* add command to generate bind params ([3c7992b](https://github.com/H3rmt/hyprswitch/commit/3c7992b8f406675469091e5c86c2cab6f118f31b))
* add gui restart on monitor connected / disconnected ([ca8ddec](https://github.com/H3rmt/hyprswitch/commit/ca8ddec353714cc5dede71fcc40d430bf5838ccf))
* add systemd restart if out of sync versions ([b51bdd1](https://github.com/H3rmt/hyprswitch/commit/b51bdd1b07aa9e140bc2cefea5695359768a9c62))
* added launching animation to launcher ([27c88e6](https://github.com/H3rmt/hyprswitch/commit/27c88e671c034b22d5e13a36de4ca23dd1025944))
* added more generation to generate command ([c22cc1e](https://github.com/H3rmt/hyprswitch/commit/c22cc1e14392b8ffcbd0450745539b3c026e9f0c))
* added new --submap argument, switched to json serialise ([0179d39](https://github.com/H3rmt/hyprswitch/commit/0179d39ecd2b2a86fcfc8628d617126d9386ec26))
* check for version in hyprland version command ([cf64356](https://github.com/H3rmt/hyprswitch/commit/cf6435657ee3a217012e98f21cbbfbb6d6df3936))
* hide picture if client to small ([4410bb8](https://github.com/H3rmt/hyprswitch/commit/4410bb80941304be0067b2cf0bcdac8be6ce7f6f))
* improve icon detection for flatpak apps ([f6ce8f7](https://github.com/H3rmt/hyprswitch/commit/f6ce8f7575640fcfe6593645da07e8450714cd3a))
* launch items in launcher when clicking ([4c56402](https://github.com/H3rmt/hyprswitch/commit/4c56402059e20b57604fd186a9d8d6ca0bf40391))
* show exec in launcher ([c7900f5](https://github.com/H3rmt/hyprswitch/commit/c7900f50bf258a81dd14d1800e8cb2f383491150))


### Bug Fixes

* add env var to control show exec in launcher ([12b561d](https://github.com/H3rmt/hyprswitch/commit/12b561d3820c55fe3dd80b6e961901819a68ff5f))
* added LAUNCHER_ANIMATE_LAUNCH_TIME to config ([8f4c46a](https://github.com/H3rmt/hyprswitch/commit/8f4c46a549583c4cfbdcc6551e9264ac685da0a4))
* added new version check command ([32f5d6f](https://github.com/H3rmt/hyprswitch/commit/32f5d6fec4db5616f9765ab170b89f1277682ef4))
* detach children ([397d6cc](https://github.com/H3rmt/hyprswitch/commit/397d6cc0a7cab3093bfa5e03fe79cc7ab049d167))
* https://github.com/H3rmt/hyprswitch/issues/115 (added Overflow::Hidden to overlay) ([3a02f88](https://github.com/H3rmt/hyprswitch/commit/3a02f887b6e6114f5d353c434134dea4cdb2307b))
* launcher layer is now KeyboardMode::Exclusive ([8dd04b9](https://github.com/H3rmt/hyprswitch/commit/8dd04b9d403c42c3f947348845e1d91911c5997e))
* only move windows down of launcher is active ([66f6e5b](https://github.com/H3rmt/hyprswitch/commit/66f6e5bc4303189237cf7403ca64f16e0cc46ced))
* only show Notification when not in debug(dev) mode ([f6ce8f7](https://github.com/H3rmt/hyprswitch/commit/f6ce8f7575640fcfe6593645da07e8450714cd3a))
* re-added bincode as transport ([57b40b7](https://github.com/H3rmt/hyprswitch/commit/57b40b7b887a8367b424ceee987c9b13cbc697ea))
* remove entries from desktop file cache map if no icon was found ([3db8292](https://github.com/H3rmt/hyprswitch/commit/3db8292ac7c587a14ae5fb8a853d956b6332fb2c))

## [3.2.5](https://github.com/H3rmt/hyprswitch/compare/v3.2.4...v3.2.5) (2025-01-08)


### Bug Fixes

* -v and -vv work again, added -q for quiet output, fix [#104](https://github.com/H3rmt/hyprswitch/issues/104) ([0b09905](https://github.com/H3rmt/hyprswitch/commit/0b09905fcba41e1d499aa435b563e81491a14a68))
* add REMOVE_HTML_FROM_WORKSPACE_NAME flag ([41f572f](https://github.com/H3rmt/hyprswitch/commit/41f572f375a1f8bd4fca0657538f594ffa7a07ca))

## [3.2.4](https://github.com/H3rmt/hyprswitch/compare/v3.2.3...v3.2.4) (2025-01-08)


### Bug Fixes

* added new logging (tracing) ([330b11f](https://github.com/H3rmt/hyprswitch/commit/330b11f854708133ef8b1a1c7113a60cdf4d637a))
* change changelog generation ([e93dd60](https://github.com/H3rmt/hyprswitch/commit/e93dd602073b7df1e2250c18af249ea840eda789))
* set active before waiting for GUI update ([82a470e](https://github.com/H3rmt/hyprswitch/commit/82a470eb9abc30228c870468260f1976671ae108))
* trace data from collect_data after enabled has been set ([dbfc1b2](https://github.com/H3rmt/hyprswitch/commit/dbfc1b211024854c72597a1dee576dc14965f7d0))

## [3.2.3](https://github.com/H3rmt/hyprswitch/compare/v3.2.2...v3.2.3) (2025-01-05)


### Bug Fixes

* ci adding component to tag ([e1c16e8](https://github.com/H3rmt/hyprswitch/commit/e1c16e802f4eaae7eb96485e64f9ee8502974e75))
* ci adding component to tag ([759e711](https://github.com/H3rmt/hyprswitch/commit/759e7111a1f80ecaae96e4b29d21f85d2f346282))
* fix some examples ([87e8868](https://github.com/H3rmt/hyprswitch/commit/87e8868c4336de653c39bbf4af406374b0068b0b))
* prevent poisoned lock when switching execs in empty list ([33a4d6b](https://github.com/H3rmt/hyprswitch/commit/33a4d6b55367122d61f239ebc29ce5dac0634654))
* reposition windows when using launcher ([7da33e3](https://github.com/H3rmt/hyprswitch/commit/7da33e34194fdde08e6696a8bb77999d6ea56a9e))
* switch default terminal order ([4bd2fe9](https://github.com/H3rmt/hyprswitch/commit/4bd2fe9e3cfdc385620f5c62eca8c7e48487a4d1))

## [3.2.2](https://github.com/H3rmt/hyprswitch/compare/v3.2.1...v3.2.2) (2025-01-04)


### Bug Fixes

* fix env loading ([979ade0](https://github.com/H3rmt/hyprswitch/commit/979ade08f72e9e2212966878a8e301f603f8518b))
* fixed launcher keybinds ([77dfb24](https://github.com/H3rmt/hyprswitch/commit/77dfb24fad22ed51e7b5b44c1a03132b6d05592e))
* set GUI layer namespace to `hyprswitch` ([5649316](https://github.com/H3rmt/hyprswitch/commit/5649316f06c305a2766d6922b92d3522516eacff))

## [3.2.1](https://github.com/H3rmt/hyprswitch/compare/v3.2.0...v3.2.1) (2025-01-03)


### Bug Fixes

* change order of sources for icon loading ([0570764](https://github.com/H3rmt/hyprswitch/commit/05707642a0e5638169fbc65c8f5919e1fb14bd9e))
* fix icons for launcher ([af7b59b](https://github.com/H3rmt/hyprswitch/commit/af7b59b94b74e8859d791d564f1bbc731a71b3b2))
* prevent usage of non-mod keys for mod reverse ([0570764](https://github.com/H3rmt/hyprswitch/commit/05707642a0e5638169fbc65c8f5919e1fb14bd9e))
