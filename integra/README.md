## Usage
* To start a localnet: `docker-compose up -d localnet`
* Then, to run the integration test: `docker-compose run compare`.

## Contents
* Besides the integration test, this folder contains
  the foundations of three utilities:
  * `integra`, which ingests contract schema and
    generates a JS wrapper with the right methods
  * `itslit`, a "literate programming" tool which
    extracts Markdown from code comments
    and renders it side-by-side with the code as HTML.
  * `justsayit`, a tracing/logging helper
    which uses arbitrary hashtags
    instead of pre-defined categories.
