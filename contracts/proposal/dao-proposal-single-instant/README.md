# dao-proposal-single-instant

[![dao-proposal-single-instant on crates.io](https://img.shields.io/crates/v/dao-proposal-single-instant.svg?logo=rust)](https://crates.io/crates/dao-proposal-single-instant)
[![docs.rs](https://img.shields.io/docsrs/dao-proposal-single-instant?logo=docsdotrs)](https://docs.rs/dao-proposal-single-instant/latest/dao_proposal_single_instant/)

A proposal module for a DAO DAO DAO which supports simple "yes", "no",
"abstain" voting. Proposals may have associated messages which will be
executed by the core module upon the proposal being passed and
executed.

Votes can be cast for as long as the proposal is not expired. In cases
where the proposal is no longer being evaluated (e.g. met the quorum and
been rejected), this allows voters to reflect their opinion even though 
it has no effect on the final proposal's status.

For more information about how these modules fit together see
[this](https://github.com/DA0-DA0/dao-contracts/wiki/DAO-DAO-Contracts-Design)
wiki page.

For information about how this module counts votes and handles passing
thresholds see
[this](https://github.com/DA0-DA0/dao-contracts/wiki/A-brief-overview-of-DAO-DAO-voting#proposal-status)
wiki page.

## Undesired behavior

The undesired behavior of this contract is tested under `testing/adversarial_tests.rs`.

In general, it should cover:
- Executing unpassed proposals
- Executing proposals more than once
- Social engineering proposals for financial benefit
- Convincing proposal modules to spend someone else's allowance

## Proposal deposits

Proposal deposits for this module are handled by the
[`dao-pre-propose-single`](../../pre-propose/dao-pre-propose-single)
contract.

## Hooks

This module supports hooks for voting and proposal status changes. One
may register a contract to receive these hooks with the `AddVoteHook`
and `AddProposalHook` methods. Upon registration the contract will
receive messages whenever a vote is cast and a proposal's status
changes (for example, when the proposal passes).

The format for these hook messages can be located in the
`proposal-hooks` and `vote-hooks` packages located in
`packages/proposal-hooks` and `packages/vote-hooks` respectively.

To stop an invalid hook receiver from locking the proposal module
receivers will be removed from the hook list if they error when
handling a hook.

## Revoting

The proposals may be configured to allow revoting.
In such cases, users are able to change their vote as long as the proposal is still open.
Revoting for the currently cast option will return an error.
