# Contributing to truce

Patches, bug reports, and feature requests are welcome.

## Code submissions and the dual-license contributor grant

truce ships under the dual license in `LICENSE`:

- The Author License — **Apache License, Version 2.0**
  (`LICENSE-APACHE`) — granted freely to plug-in authors, end-user
  audio software, **and** free, OSI-licensed, non-commercial
  framework / SDK / developer-tool projects built on top of truce
  (the Section 2.1 exemption).
- A Framework License granted only by separate written permission
  from the project maintainers, for **commercial** audio-plug-in
  frameworks built on truce — anything sold, subscription-gated,
  dual-licensed commercially, or bundled into a paid product.

For the dual-license model to work, code you contribute needs to be
licensable on both sides.

**By opening a pull request, issue patch, or other code contribution
to this repository, you agree that:**

1. You wrote the contribution yourself, or you have the legal right
   to submit it under the terms below.

2. Your contribution is licensed to the truce project and to all
   downstream recipients under the **Apache License, Version 2.0**
   (`LICENSE-APACHE`). This is identical to the standard "Inbound =
   Outbound" Apache 2.0 contribution rule per the Apache License
   §5 — your patch flows to users on the same terms as the rest of
   the project.

3. You grant the truce project the additional right to include your
   contribution under any Framework License the project grants under
   Section 2 of `LICENSE`, on whatever terms the project negotiates.

4. You retain copyright in your contribution. This grant does not
   transfer ownership; it grants the project the licensing rights
   needed to make the dual-license model work.

You do not need to sign a separate CLA document — opening the PR is
the agreement. If you can't agree to the above for any reason (an
employer claims rights to the code, you're unsure who owns it, etc.),
note that in the PR and a maintainer will work with you to sort it
out before merging.

If you're contributing on behalf of an employer or another legal
entity, please confirm in the PR that the entity authorizes the
above grant.

## Code quality

- `cargo clippy --workspace --all-targets -- -D warnings` must be
  clean.
- `cargo fmt --all --check` must be clean.
- `cargo test --workspace --lib` must pass.
- New crates / modules need rustdoc-warning-free `cargo doc
  --workspace --no-deps`.
- Comments explain **why**, not what. Don't reference past audits or
  PRs by name — those rot.
- Don't add error handling, fallbacks, or validation for scenarios
  that can't happen. Trust internal code.
