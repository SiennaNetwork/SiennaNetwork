## Contributing

We use the [gitflow](https://danielkummer.github.io/git-flow-cheatsheet/) workflow [this is also helpful](https://gist.github.com/JamesMGreene/cdd0ac49f90c987e45ac).
* Development of features happens in branches made from **dev** called feature/<the-feature> like feature/add-token-rewards.
* When development is finished a pull request to **dev** is created. At least one person has to review the PR and when everything is fine the PR gets merged.
* To make a new release create a release branch called release/X.X.X, also bump the version number in package.json in this branch.
* Create a PR to `main` which then also has to be accepted.
* Create a tag for this version and push the tag.
* Also merge back the changes (like the version bump) into `dev`.
  
### Rules
- Use `rebase` instead of `merge` to update your codebase, except when a PR gets included in a branch.
- Use meaningfull descriptions and titles in commit messages.
- Explain what you did in your PRs, add images whenever possible for showing the status before/after the change visually. 