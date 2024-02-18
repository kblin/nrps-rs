# NRPS-rs: A Rust reimplementation of NRPSPredictor2

This is an (almost) feature-for-feature reimplementation of NRPSPredictor2 in Rust.
On the SVM side of things, it loads the same mdl files with feature vectors and produces the same scores.
The Stachelhaus tables have been updated with all the signature/substrate mappings from the MIBiG 3.1 release.
The output text format is slightly different, as NRPS-rs adds an additional 8 Å signature lookup to
differentiate between results with identical Stachelhaus signatures.

## Installation

NRPS-rs is available via `cargo`, so the easiest way to install it is to run

```bash
cargo install nrps-rs
```

Alternatively, you can clone the git repository and run

```bash
cargo build -r
```

and then copy the resulting binary from `target/release/nrps-rs` into your `$PATH`.

## Data

In order to actually run NRPS-rs, you'll need to provide a Stachelhaus signature file and SVM model files.
You can fetch the ones antiSMASH uses from https://dl.secondarymetabolites.org/releases/stachelhaus/1.1/
and https://dl.secondarymetabolites.org/releases/nrps_svm/2.0/

NRPS-rs looks in `$PWD/data/models` by default, but you can set alternative locations using the `--model-dir`
(and `--stachelhaus-signatures`) parameters or the config file.

## Configuration

NRPS-rs can be configured via command line parameters or a config file. By default,
NRPS-rs looks for a file named `nrps.toml` in the current working directory, this can
be overridden by the `--config` parameter.

## Running NRPS-rs

To run NRPS-rs, you need to provide an input file containing the 8 Å active site signature
of the adenylation domain(s) you want to predict, with one line per A domain containing the
34 AA signature and an identifier for the domain, separated by a tab.

### Example

This example assumes you have the antiSMASH models and signatures installed as described above.
We use `--skip-v3` because the antiSMASH model set doesn't provide the experimental third generation
models but sticks with the original NRPSPredictor2 ones.

```bash
echo -e "LDASFDASLFEMYLLTGGDRNMYGPTEATMCATW\tbpsA" > example.sigs
nrps-rs --skip-v3 example.sigs
Name	8A signature	Stachelhaus signature	Full Stachelhaus match	AA10 score	AA10 signature matched	AA34 score	Stachelhaus	ThreeClusterV2	LargeClusterV2	SmallClusterV2	SingleV2	LargeClusterV1	SmallClusterV1
bpsA	LDASFDASLFEMYLLTGGDRNMYGPTEATMCATW	DAFYLGMMCK	Leu/Leu/Leu	1.00/1.00/1.00	DAFYLGMMCK/DAFYLGMMCK/DAFYLGMMCK	1.00/0.94/0.88	Leu(1.00)	hydrophobic-aliphatic(1.03)	N/A	val,leu,ile,abu,iva(0.21)	leu(0.43)	gly,ala,val,leu,ile,abu,iva(1.00)	val,leu,ile,abu,iva(1.00)
```

## License

NRPS-rs is an open source tool available under the GNU Affero General Public
License version 3.0 or greater. See the [`LICENSE.txt`](LICENSE.txt) file for
details.
