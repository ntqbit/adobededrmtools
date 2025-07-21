# Adobe DeDRM tools

This is a command-line tool that lets you turn `.acsm` into plain EPUB without using Adobe Digital Editions.


## Installation

You need [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed. Then run:

```bash
cargo install --git https://github.com/ntqbit/adobededrmtools --bin adobededrmtools
```

This will install the `adobededrmtools` binary.

## Usage

```
Usage: adobededrmtools [OPTIONS] --acsm <ACSM>

Options:
      --acsm <ACSM>        Path to .acsm file
      --account <ACCOUNT>  Path to JSON account file [default: account.json]
      --out <OUT>          Path to directory to write the output resources to [default: .]
  -h, --help               Print help
  -V, --version            Print version
```

To get EPUB from `.acsm` run:
```bash
adobededrmtools --acsm /path/to/book.acsm
```

On success, the tool writes one or more files named `resource_{n}.epub` (for example, `resource_1.epub`) into the output directory (current directory by default).

When you run it for the first time, it creates an anonymous Adobe account and saves its credentials to `account.json` in the working directory. Use `--account /path/to/account.json` to choose a different location. On subsequent runs, the tool will reuse the saved credentials instead of creating a new account.

## License

This project is licensed under [LGPL-3.0](./LICENSE)

## Credits

This implementation was inspired by open-source implementation of Adobe's ADEPT protocol [libgourou](https://forge.soutade.fr/soutade/libgourou). Many thanks to its authors for making their work available.

## Related works

- https://forge.soutade.fr/soutade/libgourou
- https://github.com/Leseratte10/acsm-calibre-plugin
- https://github.com/apprenticeharper/DeDRM_tools