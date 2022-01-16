# Contribution guidelines

We use the [gitflow](https://danielkummer.github.io/git-flow-cheatsheet/) workflow.
[See this Gist for more info.](https://gist.github.com/JamesMGreene/cdd0ac49f90c987e45ac).

* Develop features in branches made from **dev**,
  called `feature/<the-feature>` e.g. `feature/add-token-rewards`.

* When development is finished, create a pull request to **dev**.
  At least one person should review the PR. When everything is fine, the PR gets merged.

* To make a new release, create a release branch called `release/X.X.X`,
  also bump the version number in `package.json` in that branch.

* Create a PR to `main` which then also has to be accepted.

* Create a tag for the version and push the tag.

* Also merge back the changes (like the version bump) into `dev`.

## When using Git:

- Use `rebase` instead of `merge` to update your codebase,
  except when a PR gets included in a branch.

- Use meaningful descriptions and titles in commit messages.

- Explain what you did in your PRs, add images whenever possible
  for showing the status before/after the change visually. 

## Coding style

* You are free to use **100 column lines**.
  When embedding links in comments or documentation, links that go over the 100 column limit are
  welcome to remain on the same line, as long as you put a line break immediately afterwards.
  See the source of this file for an example!

* You are encouraged to use **TypeScript**.
  However, you are free to do what the Node.js runtime allows, not just what TypeScript likes.
  The [module loader](https://github.com/hackbg/ganesha) takes care of compiling TypeScript
  at runtime, so you may prefer to avoid introducing unnecessary build steps for internal modules. 

* You are free to take **unconventional approaches**, as long as you have a plan
  and the arguments to back it up. Our mission is to build a new, more fair economy -
  factory-floor innovation is most welcome.

* We encourage **high-level documentation** in the form of [literate modules](https://github.com/hackbg/ganesha),
  as well as **low-level documentation** in the form of [TSDoc](https://tsdoc.org/) docstrings.
  If you find yourself writing a comment that just restates what the code does,
  consider moving to a higher level and **documenting the control flow** instead.

* In terms of formatting, we prioritize **consistency over uniformity**.
  This means sloppy formatting and auto-formatting are considered equally bad taste, and every
  contributor is free to format their code in the way that is most accessible to their senses -
  while taking into consideration the purpose and structure of the code in question,
  as well as the preferences of other contributors who work on the same area of the code. 

<div align="center">

```
"A foolish consistency is the hobgoblin of little minds"
- Ralph Waldo Emerson
```

</div>
