# Changelog

## [1.3.2](https://github.com/narnaud/use/compare/v1.3.1...v1.3.2) (2025-11-25)


### Documentation

* Fix typos ([85e4640](https://github.com/narnaud/use/commit/85e46408de0d6912686e68c289489f078856b05e))

## [1.3.1](https://github.com/narnaud/use/compare/v1.3.0...v1.3.1) (2025-11-21)


### Bug Fixes 🐞

* **print:** Improve readability of the print command ([0e5058b](https://github.com/narnaud/use/commit/0e5058bfd96e50cc4e1df847d14962e724bcb4cd))


### Other

* Update dependencies ([d9caaa0](https://github.com/narnaud/use/commit/d9caaa01903ed8d9f203746fd36aa43cc00b839d))

## [1.3.0](https://github.com/narnaud/use/compare/v1.2.2...v1.3.0) (2025-11-16)


### Features ✨

* Change config file path to $HOME/.config/use.yaml ([0a89acd](https://github.com/narnaud/use/commit/0a89acda4c8a6ebe4e1744061c878dad61281dee))


### Bug Fixes 🐞

* **cmd:** Fix go to use `chdir` instead of `cd` ([8b6baac](https://github.com/narnaud/use/commit/8b6baac74b42a8ae21a3f94668b1c7f6c75ba844))
* Remove double `:` in warning and error messages ([812b853](https://github.com/narnaud/use/commit/812b8532cad07e9ae256c28118e2d06a23828821))

## [1.2.2](https://github.com/narnaud/use/compare/v1.2.1...v1.2.2) (2025-10-07)


### Bug Fixes 🐞

* **cmd:** Fix detection of flags in init script ([4302cc9](https://github.com/narnaud/use/commit/4302cc9c8569637056edd302ed17b7ff81f617b7))


### Other

* update pre-commit hooks ([e56420f](https://github.com/narnaud/use/commit/e56420f52404b3d0f01d3e70de2df4d8f8036e5b))

## [1.2.1](https://github.com/narnaud/use/compare/v1.2.0...v1.2.1) (2025-09-18)


### Bug Fixes

* Allow changing disk with cmd ([e3f6d41](https://github.com/narnaud/use/commit/e3f6d413e0d6dd3f0c86ecfc9d09e1c4481a56e9))

## [1.2.0](https://github.com/narnaud/use/compare/v1.1.0...v1.2.0) (2025-09-12)


### Features

* Add env variable substitution ([939c195](https://github.com/narnaud/use/commit/939c195de2a44c4daad2dd05bbd7c58bc4dbafe9))
* Resolve dependencies between environment variables ([d26271c](https://github.com/narnaud/use/commit/d26271c1f657b6fe265d2396401be5969c859902))


### Bug Fixes

* **cmd:** Add missing commands to completion ([1fad1e9](https://github.com/narnaud/use/commit/1fad1e9251e21b75b042e0daf53bcda80d6a529f))
* Order of dependencies is now correct ([c0eeb0b](https://github.com/narnaud/use/commit/c0eeb0b82f22852c2010f62325f5775bfa2da235))
* **powershell:** Add missing commands to completion ([f796225](https://github.com/narnaud/use/commit/f79622596a73911ca226111f7e1feed14548f676))

## [1.1.0](https://github.com/narnaud/use/compare/v1.0.0...v1.1.0) (2025-09-09)


### Features

* Add config file handling ([99b0440](https://github.com/narnaud/use/commit/99b044071352ffdfdf0ac1f20dbde2bdb5d3d08f))
* Add finished line to the output ([facf312](https://github.com/narnaud/use/commit/facf312e8c88e025442582a34941b42f3e14f3fa))
* Add print command ([8e2a7bc](https://github.com/narnaud/use/commit/8e2a7bceb9cd5a02e74271256743c3a9d48ca735))


### Bug Fixes

* Fix some rust clippy issues ([fc2ef8f](https://github.com/narnaud/use/commit/fc2ef8fe663e1edde30bdef0134678abc630c3c1))
* fold environment with a specific shell ([21ea69d](https://github.com/narnaud/use/commit/21ea69d62ed9584f838403094d628e90f4dcdb06))

## [1.0.0](https://github.com/narnaud/use/compare/v0.3.2...v1.0.0) (2025-08-23)


### ⚠ BREAKING CHANGES

* use is now generating the shell commands
* Change settings cli options
* Integrate shell initialisation in the executable

### Features

* Integrate shell initialisation in the executable ([0adebfd](https://github.com/narnaud/use/commit/0adebfd664e6ad064c44019422876e5c89c3a9e0))


### Bug Fixes

* Add shell integration scripts into package ([06e30b6](https://github.com/narnaud/use/commit/06e30b605fbd8b7d26266d659e3dfbeae006d6f9))
* Another fix, this time the executable name... ([9658269](https://github.com/narnaud/use/commit/965826974ba62499939104a1e03af24b9326f68c))


### Code Refactoring

* Change settings cli options ([f6bfb18](https://github.com/narnaud/use/commit/f6bfb1863bffdb880f8a572ff68162733a52a7e4))
* use is now generating the shell commands ([b23b39f](https://github.com/narnaud/use/commit/b23b39f0b5b5779487a85a68b44d1ca3c6acf18d))

## [0.3.2](https://github.com/narnaud/use/compare/v0.3.1...v0.3.2) (2025-04-22)


### Bug Fixes

* **clink:** Do not add empty lines ([cc78257](https://github.com/narnaud/use/commit/cc782578b85302b041d906d968503ee6a92003c0))

## [0.3.1](https://github.com/narnaud/use/compare/v0.3.0...v0.3.1) (2025-03-29)


### Bug Fixes

* **posh:** Fix powershell manifest ([79b61c3](https://github.com/narnaud/use/commit/79b61c35d58eea70720b94b44f910ad17c68fd01))

## [0.3.0](https://github.com/narnaud/use/compare/v0.2.0...v0.3.0) (2025-03-29)


### Features

* Add a new settings to change the terminal title or not ([1690486](https://github.com/narnaud/use/commit/1690486b20d0f702ea0be79f7e416ee0c36311a8))
* Add pattern matching for environment ([428a596](https://github.com/narnaud/use/commit/428a596db003d7c83ff001a26302ce1782a8bec2))
* Move to YAML for the config file ([f9a8e7e](https://github.com/narnaud/use/commit/f9a8e7ec30c930d579bd15dc65abca61717cf546))
* **posh:** Add Powershell integration via a module ([c62ec9e](https://github.com/narnaud/use/commit/c62ec9ee27e430c86147eecc7f98b6711a444079))
* Use partial keys for environment ([367b101](https://github.com/narnaud/use/commit/367b101e90ddcc0691455d3421e411ad936d987d))


### Bug Fixes

* **clink:** Use console.settitle instead of collaing TITLE... ([32c95ab](https://github.com/narnaud/use/commit/32c95ab094f51b36a7a05ab0131fbcd3bfba2a19))
* Remove unused dependency ([fa6bff4](https://github.com/narnaud/use/commit/fa6bff427d523e0bc5672a91efcadd6bcbca5c12))

## [0.2.0](https://github.com/narnaud/use/compare/v0.1.0...v0.2.0) (2025-03-19)


### Features

* Add some coloring and better console output ([61ed79a](https://github.com/narnaud/use/commit/61ed79af97da383184a2a4112b8aea4788671cdc))
* Add the final information ([9e1cafa](https://github.com/narnaud/use/commit/9e1cafa091bb912c8c8375ed05bde6abd1ee7a1d))
* **clink:** Add clink completion script ([976784e](https://github.com/narnaud/use/commit/976784e93e073a19f6945b2116a3fea1866c61d7))
* **clink:** Add clink integration ([bb831d5](https://github.com/narnaud/use/commit/bb831d57112117379e6d40270e3f2d412ad1ab55))
* **clink:** Set the terminal title ([ec4a047](https://github.com/narnaud/use/commit/ec4a047dde79f17359fcdbcda9ebb44881466b45))
* **cmd:** Set the terminal title ([73315bb](https://github.com/narnaud/use/commit/73315bbca1faeb4e8f8e147b5f03893d1c65be19))


### Bug Fixes

* Fix example config file json ([1bd7fe7](https://github.com/narnaud/use/commit/1bd7fe7b4df12803940c9277a05e8f28f17a9663))

## 0.1.0 (2025-03-19)


### Features

* Add batch file to use an env. ([6a1b8a6](https://github.com/narnaud/use/commit/6a1b8a6ac5eaf12bd8016eb301c244ccd9d7d34a))
* Add Cargo.lock to the repository ([c239b6b](https://github.com/narnaud/use/commit/c239b6bab1d137415ccf31c42a997fd829bfc458))
* Add logger using log crate ([c875977](https://github.com/narnaud/use/commit/c875977265438016bcc8f488687d396880645889))
* Check the config file ([41b2c25](https://github.com/narnaud/use/commit/41b2c250b434d070958ae819f32de1a44c471053))
* Handle command line paramters ([3a4e7a7](https://github.com/narnaud/use/commit/3a4e7a70652ceda37f6d8875d6c34d72a823a025))
* Merge environments before printing ([63dbaf9](https://github.com/narnaud/use/commit/63dbaf963ed8a12a3b3bc6f31700ae3c3c6bfd34))
* Print the resulting environment ([75a9faa](https://github.com/narnaud/use/commit/75a9faa30578a84624ea7dc6053f5dbf2119f4bf))
* Read the config file ([868c1e3](https://github.com/narnaud/use/commit/868c1e355fbf68087bcd3f9de297a5d0e4e2e5d4))
* Return the list of environment for a given name ([bd2e62a](https://github.com/narnaud/use/commit/bd2e62aa42e1ca166f347dd8b018d85dc4e099d9))


### Bug Fixes

* Fix cmd script ([d90beab](https://github.com/narnaud/use/commit/d90beab956c958e2b8b3503a3a27316edec4b476))
* Fix package name ([4ec78dd](https://github.com/narnaud/use/commit/4ec78ddda7347b28e38e2165ce4a7a063a9b878e))
