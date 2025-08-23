# Pull Request Template

## ✍️ Summary (Required)
<!-- 
Use a Conventional Commit style summary: 
<type>(<scope>): <short summary>

Examples:
- feat(cli): add --latest flag to fetch the most recent mirrors
- fix(parser): handle empty mirrorlist gracefully
- docs(readme): update usage examples
- refactor(config): simplify mirror selection logic
- test(mirror): add integration tests for https mirrors
-->

## 📖 Details
<!-- 
Optional: Provide reasoning, background, or extended context. 
This will appear in the squash commit body and changelog.
-->

## 🔄 Type of Change
<!-- Mark with an [x] -->
- [ ] ✨ feat: A new feature
- [ ] 🐛 fix: A bug fix
- [ ] 📚 docs: Documentation only changes
- [ ] 🛠 refactor: Code change that neither fixes a bug nor adds a feature
- [ ] 🎨 style: Formatting, missing semi-colons, etc
- [ ] ✅ test: Adding or correcting tests
- [ ] ⚙️ chore: Maintenance tasks (build, deps, CI, etc)
- [ ] ⚠️ BREAKING CHANGE: Backward-incompatible change

## 🚨 Breaking Changes
<!-- 
If this PR introduces a breaking change, describe it here.
Example:
- feat(cli): rename --country to --region
-->

## ✅ Checklist

- [ ] Code is formatted with `cargo fmt`
- [ ] Code passes lint checks with `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
- [ ] All tests pass with `cargo test`
- [ ] Coverage of tests stays over 80% `cargo tarpaulin`
- [ ] Dependency check succeeds with `cargo deny check`
- [ ] Documentation updated (if applicable)
- [ ] Added/updated tests for new functionality (if applicable)
