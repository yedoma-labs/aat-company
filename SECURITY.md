# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Scope

`aat-company` is a pure text library: no `unsafe` code (`#![forbid(unsafe_code)]`), no network, no dependencies. The relevant risks are:

- **Resource use on hostile input.** `compare` is linear in the name length (Jaro-Winkler is quadratic in the length of the longer name, so cap the length of untrusted names, for example at 200 characters). `dedupe` compares only names inside the same block; many names that share a block (for example thousands starting with the same word) still cost quadratic time in that block.
- **Wrong merges.** A match is a heuristic. Do not merge legal or financial records on `Likely` or lower without review, and never on names alone when a registry identifier is available.

## Reporting a Vulnerability

**Do not** open a public issue for security vulnerabilities. Use a private GitHub Security Advisory on this repository, or email yedoma-labs@proton.me. Include a description, steps to reproduce and the impact. We acknowledge within 48 hours.
