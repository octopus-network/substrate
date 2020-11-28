
# Substrate Bftml Branch

This is the bftml branch of substrate.

BFTML, which is the abbreviation of BFT Middle Layer. It is the middle layer in substrate consensus framework, lying below concrete consensus engine, and above the substrate raw consensus framework.

## Advantages

- Bftml reduces amounts of complexity of porting a new BFT-like consensus protocol/engine to substrate, it was very hard to program on raw consensus framework (if you ever tried it, you could understand)
- Bftml exposes only 5 channels/interfaces for upper layer consensus engine, clearly
- Bftml provides the possibility that a consensus engine written in a different language (such as go, c++, java, or python) could be integrated with substrate
- So Bftml could make substrate a warm nursery for any new bft-like consensus innovations.

## Status

PoC0, before alpha.

----





# Substrate &middot; [![GitHub license](https://img.shields.io/badge/license-GPL3%2FApache2-blue)](LICENSE) [![GitLab Status](https://gitlab.parity.io/parity/substrate/badges/master/pipeline.svg)](https://gitlab.parity.io/parity/substrate/pipelines) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](docs/CONTRIBUTING.adoc)

<p align="center">
  <img src="/docs/media/sub.gif">
</p>


Substrate is a next-generation framework for blockchain innovation 🚀.

## Trying it out

Simply go to [substrate.dev](https://substrate.dev) and follow the 
[installation](https://substrate.dev/docs/en/knowledgebase/getting-started/) instructions. You can 
also try out one of the [tutorials](https://substrate.dev/en/tutorials).

## Contributions & Code of Conduct

Please follow the contributions guidelines as outlined in [`docs/CONTRIBUTING.adoc`](docs/CONTRIBUTING.adoc). In all communications and contributions, this project follows the [Contributor Covenant Code of Conduct](docs/CODE_OF_CONDUCT.md).

## Security

The security policy and procedures can be found in [`docs/SECURITY.md`](docs/SECURITY.md).

## License

- Substrate Primitives (`sp-*`), Frame (`frame-*`) and the pallets (`pallets-*`), binaries (`/bin`) and all other utilities are licensed under [Apache 2.0](LICENSE-APACHE2).
- Substrate Client (`/client/*` / `sc-*`) is licensed under [GPL v3.0 with a classpath linking exception](LICENSE-GPL3).

The reason for the split-licensing is to ensure that for the vast majority of teams using Substrate to create feature-chains, then all changes can be made entirely in Apache2-licensed code, allowing teams full freedom over what and how they release and giving licensing clarity to commercial teams.

In the interests of the community, we require any deeper improvements made to Substrate's core logic (e.g. Substrate's internal consensus, crypto or database code) to be contributed back so everyone can benefit.
