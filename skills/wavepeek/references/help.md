# Help command

`wavepeek` uses progressive disclosure: start with a compact reminder, then ask for more detail only when you need it.

## Help layers

Use `wavepeek -h` for compact top-level lookup help. This layer is intentionally short and points you to the deeper layers.

Use `wavepeek --help` for detailed top-level reference help. This layer explains general conventions and lists the available command families.

Use `wavepeek help <command-path...>` for detailed help on a specific command or nested command. For example:

    wavepeek help change
    wavepeek help extract axi
    wavepeek help skill

You can also ask a command directly for detailed help with `wavepeek <command> --help` or `wavepeek <command-path...> --help`.

## Where narrative docs fit

Generated help is the authority for exact syntax, flags, defaults, and required arguments. Use this package's relative links for narrative guidance, workflows, troubleshooting, and stable semantic references.

Use `wavepeek skill <DIRECTORY>` to extract the complete, version-matched package into a new or empty directory.
