# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.8](https://github.com/nagauta/gig/compare/v0.1.7...v0.1.8) - 2026-03-26

### Added

- add command generator support and distinct icon for command completions
- enhance completion system with generator kind support and distinct file icon
- add support for git checkout subcommand and enhance completion options

### Other

- streamline command output handling in run_generator function

## [0.1.7](https://github.com/nagauta/gig/compare/v0.1.6...v0.1.7) - 2026-03-26

### Added

- enhance shell completion by handling trailing spaces for improved token completion
- xxx

### Other

- update pull request template to use English headings for better accessibility

## [0.1.6](https://github.com/nagauta/gig/compare/v0.1.5...v0.1.6) - 2026-03-21

### Added

- add interactive completion picker using TUI

### Other

- clean up App initialization and formatting in TUI
- remove homebrew installer from dist-workspace.toml

## [0.1.5](https://github.com/nagauta/gig/compare/v0.1.4...v0.1.5) - 2026-03-21

### Added

- *(ci)* use GitHub App token for release-plz to trigger cargo-dist

## [0.1.4](https://github.com/nagauta/gig/compare/v0.1.3...v0.1.4) - 2026-03-21

### Fixed

- *(ci)* run release-pr after release to prevent duplicate PRs

### Other

- clarify PR template instructions to include both title and body in English

## [0.1.3](https://github.com/nagauta/gig/compare/v0.1.2...v0.1.3) - 2026-03-21

### Added

- add release job to release-plz workflow
- add lefthook configuration for pre-push commands and refactor code for improved readability
- add pull request template for better contribution guidelines
- update project configuration and CI workflows
- add commit and pull request skills documentation

### Fixed

- disable crates.io publish in release-plz
- regenerate release.yml with dist init to fix CI

### Other

- release v0.1.2
- update action versions in release workflow for consistency and stability
- translate pr skill description to English
- update GitHub Actions workflows to improve release process and specify action versions
- update GitHub Actions to use specific versions of checkout, upload-artifact, and download-artifact actions
- Add README and MIT license
- Fix zsh completion descriptions and embed binary path
- Add install/uninstall commands with shell config management
- Add docker and npm completion specs
- Add shell integration and complete subcommand
- Add completion spec parser with TDD
- Rename fig to gig
- Initialize Rust project with CLI skeleton
- Add MVP definition for fig CLI

## [0.1.2](https://github.com/nagauta/gig/compare/v0.1.1...v0.1.2) - 2026-03-21

### Added

- add release job to release-plz workflow
- add lefthook configuration for pre-push commands and refactor code for improved readability
- add pull request template for better contribution guidelines
- update project configuration and CI workflows
- add commit and pull request skills documentation

### Fixed

- disable crates.io publish in release-plz
- regenerate release.yml with dist init to fix CI

### Other

- update action versions in release workflow for consistency and stability
- translate pr skill description to English
- update GitHub Actions workflows to improve release process and specify action versions
- update GitHub Actions to use specific versions of checkout, upload-artifact, and download-artifact actions
- Add README and MIT license
- Fix zsh completion descriptions and embed binary path
- Add install/uninstall commands with shell config management
- Add docker and npm completion specs
- Add shell integration and complete subcommand
- Add completion spec parser with TDD
- Rename fig to gig
- Initialize Rust project with CLI skeleton
- Add MVP definition for fig CLI

## [0.1.1](https://github.com/nagauta/gig/compare/v0.1.0...v0.1.1) - 2026-03-21

### Added

- add lefthook configuration for pre-push commands and refactor code for improved readability
- add pull request template for better contribution guidelines
- update project configuration and CI workflows
- add commit and pull request skills documentation

### Fixed

- regenerate release.yml with dist init to fix CI

### Other

- update action versions in release workflow for consistency and stability
- translate pr skill description to English
- update GitHub Actions workflows to improve release process and specify action versions
- update GitHub Actions to use specific versions of checkout, upload-artifact, and download-artifact actions
- Add README and MIT license
- Fix zsh completion descriptions and embed binary path
- Add install/uninstall commands with shell config management
- Add docker and npm completion specs
- Add shell integration and complete subcommand
- Add completion spec parser with TDD
- Rename fig to gig
- Initialize Rust project with CLI skeleton
- Add MVP definition for fig CLI
